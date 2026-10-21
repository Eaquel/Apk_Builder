#!/usr/bin/env python3
"""Every check this project has, run from one place.

    ./Test/Builder_Test.py                 every check that can run here
    ./Test/Builder_Test.py core strings    only the checks named
    ./Test/Builder_Test.py --list          what the names are
    ./Test/Builder_Test.py --strict        a check that cannot run here fails

A check that needs a tool this machine does not have is reported as
SKIPPED and does not fail the run, unless --strict is passed. Continuous
integration passes --strict, so a missing tool there is a failure rather
than a quiet gap.

Everything a check writes goes under Tests/, which is never committed.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
import zipfile
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEST = ROOT / "Test"
WORK = ROOT / "Tests"
CORE = ROOT / "Builder.rs"
SPEECH = ROOT / "Diagnostic_Speech.rs"
TERMINAL = ROOT / "Terminal.rs"
TERMINAL_SPEECH = ROOT / "Terminal_Speech.rs"
VIBE_CODING = ROOT / "Vibe_Coding.rs"
CORE_TESTS = TEST / "Builder_Test.rs"
TEST_DRIVER = TEST / "Builder_Test.py"
BASELINE = TEST / "Builder_Test_Baseline.json"
REFERENCE_SUITES = ["bundles", "filecheck", "tablegen-lit"]
ORACLE_SUITES = ["regex-oracle", "tablegen-oracle"]
SUITE_HOMES = {"tablegen-lit": "table_gen", "tablegen-oracle": "table_gen"}
KOTLIN_SUITES = ["kotlin-omni", "kotlin-omni-native"]
LINKER_SUITES = ["lld-elf", "lld-coff"]
ASSEMBLER_SUITES = ["mc-x86", "mc-x86-disassembler"]
RECORDING = {"on": False}
KOTLIN = ROOT / "Builder/Source/Main/Kotlin/com/apk/builder/Builder.kt"
KOTLIN_ROOT = ROOT / "Builder/Source/Main/Kotlin"
RESOURCES = ROOT / "Builder/Source/Main/res"
COMPILERS = ROOT / "Compilers"
INFRASTRUCTURE = COMPILERS / "Compiler_Infrastructure"
ANDROID = ROOT / "Android.rs"

CONFORMANCE = {
    "APK_BUILDER_REQUIRE_APKSIGNER": "1",
    "APK_BUILDER_REQUIRE_AXML_CONFORMANCE": "1",
    "APK_BUILDER_REQUIRE_IMAGE_CONFORMANCE": "1",
    "APK_BUILDER_REQUIRE_INFLATE_CONFORMANCE": "1",
}

def interface() -> str:
    """Every line of Kotlin in this application, as one text.

    The interface is more than one file -- the ceremonies are their own -- so a
    check that reads only the file the activity lives in would let a string
    nothing declares, or a native method nothing exports, through in any of the
    others.
    """
    return "\n".join(
        path.read_text(encoding="utf-8") for path in sorted(KOTLIN_ROOT.rglob("*.kt"))
    )

class Skip(Exception):
    """A check cannot run here because a tool it needs is missing."""

@dataclass
class Result:
    name: str
    state: str
    seconds: float
    detail: str = ""
    lines: list[str] = field(default_factory=list)

def sdk_root() -> Path:
    for name in ("ANDROID_HOME", "ANDROID_SDK_ROOT"):
        value = os.environ.get(name)
        if value and Path(value).is_dir():
            return Path(value)
    for guess in ("/opt/android-sdk", "/usr/local/lib/android/sdk"):
        if Path(guess).is_dir():
            return Path(guess)
    home = os.environ.get("HOME")
    if home and (Path(home) / "Android/Sdk").is_dir():
        return Path(home) / "Android/Sdk"
    raise Skip("no Android SDK on this machine")

def build_tool(name: str) -> Path:
    root = sdk_root()
    found = sorted((root / "build-tools").glob(f"*/{name}"))
    if not found:
        raise Skip(f"{name} is not in the Android build tools here")
    return found[-1]

def ndk_tool(name: str) -> Path:
    root = sdk_root()
    found = sorted(root.glob(f"ndk/*/toolchains/llvm/prebuilt/*/bin/{name}"))
    if not found:
        raise Skip(f"{name} is not in the NDK here")
    return found[-1]

def run(command: list[str] | str, *, env: dict[str, str] | None = None,
        cwd: Path = ROOT, check: bool = True) -> subprocess.CompletedProcess:
    merged = dict(os.environ)
    merged.update(env or {})
    shell = isinstance(command, str)
    done = subprocess.run(
        command, cwd=cwd, env=merged, shell=shell,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
    )
    if check and done.returncode != 0:
        tail = "\n".join(done.stdout.strip().splitlines()[-40:])
        raise AssertionError(
            f"{command if shell else ' '.join(command)} exited {done.returncode}\n{tail}"
        )
    return done

def gradle(*tasks: str) -> subprocess.CompletedProcess:
    sdk_root()
    return run([str(ROOT / "gradlew"), *tasks, "--console=plain"])

def check_layout() -> str:
    """The tree is the one this project agreed on."""
    required = [
        CORE,
        SPEECH,
        TERMINAL,
        TERMINAL_SPEECH,
        VIBE_CODING,
        CORE_TESTS,
        ROOT / "Cargo.toml",
        TEST_DRIVER,
        BASELINE,
        ROOT / "build.gradle.kts",
        ROOT / "settings.gradle.kts",
        ROOT / "gradle.properties",
        ROOT / "gradlew",
        ROOT / "Gradle/gradle-wrapper.jar",
        ROOT / "Gradle/gradle-wrapper.properties",
        KOTLIN,
        ROOT / "Builder/Source/Main/AndroidManifest.xml",
        ROOT / "Builder/Source/Main/Native/Builder.cpp",
        ROOT / "Builder/Source/Main/Native/Builder.hpp",
        ROOT / "Builder/Source/Main/Native/CMakeLists.txt",
        ROOT / "Builder/build.gradle.kts",
        ROOT / "Builder/proguard-rules.pro",
        ANDROID,
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.is_file()]
    if missing:
        raise AssertionError("missing: " + ", ".join(missing))

    compilers = sorted(path.name for path in COMPILERS.glob("*.rs"))
    expected = ["Cpp.rs", "Rust.rs"]
    if compilers != expected:
        raise AssertionError(f"Compilers holds {compilers}, expected {expected}")

    java = sorted(path.name for path in (COMPILERS / "Java").glob("*.rs"))
    wanted_java = ["Java.rs", "Java_Diagnostic.rs", "Java_Fixer.rs"]
    if java != wanted_java:
        raise AssertionError(f"Compilers/Java holds {java}, expected {wanted_java}")

    kotlin = sorted(path.name for path in (COMPILERS / "Kotlin").glob("*.rs"))
    wanted_kotlin = ["Kotlin.rs", "Kotlin_Diagnostic.rs", "Kotlin_Fixer.rs"]
    if kotlin != wanted_kotlin:
        raise AssertionError(f"Compilers/Kotlin holds {kotlin}, expected {wanted_kotlin}")

    infrastructure = sorted(path.name for path in INFRASTRUCTURE.iterdir())
    needed_infrastructure = ["Compiler_Infrastructure.rs", "Compiler_Infrastructure_Diagnostic.rs"]
    allowed_infrastructure = needed_infrastructure + [
        "TableGen.rs",
        "Armv7.rs",
        "Armv8.rs",
        "X86.rs",
        "X86_64.rs",
        "Elf.rs",
        "Pe.rs",
        "Linker.rs",
        "Tables",
        "Tables.manifest",
    ]
    absent = [name for name in needed_infrastructure if name not in infrastructure]
    foreign = [name for name in infrastructure if name not in allowed_infrastructure]
    if absent or foreign:
        raise AssertionError(
            f"Compilers/Compiler_Infrastructure holds {infrastructure}, "
            f"it needs {needed_infrastructure} and allows only {allowed_infrastructure}"
        )
    tables = INFRASTRUCTURE / "Tables"
    if tables.exists():
        held = sorted(path.name for path in tables.iterdir())
        allowed_tables = ["Armv7.bin", "Armv8.bin", "X86.bin", "X86_64.bin"]
        stray_tables = [name for name in held if name not in allowed_tables]
        if not tables.is_dir() or stray_tables:
            raise AssertionError(f"Compilers/Compiler_Infrastructure/Tables holds {held}, allowed {allowed_tables}")

    folders = sorted(path.name for path in COMPILERS.iterdir() if path.is_dir())
    wanted_folders = ["Compiler_Infrastructure", "Java", "Kotlin"]
    if folders != wanted_folders:
        raise AssertionError(f"Compilers holds the folders {folders}, expected {wanted_folders}")

    if (ROOT / "Examples").exists():
        raise AssertionError("Examples is gone from this tree")

    if (ROOT / "Old_Layout.rs").exists() or (ROOT / "Plugins").exists():
        raise AssertionError("the old Old_Layout.rs / Plugins layout is still here")

    stray = sorted(
        str(path.relative_to(ROOT))
        for path in list(ROOT.glob("*.py")) + list(TEST.glob("*.py"))
        if path != TEST_DRIVER
    )
    if stray:
        raise AssertionError(f"tests live in Test/Builder_Test.py alone, found {stray}")

    for moved in ("Builder_Test.rs", "Builder_Test.py"):
        if (ROOT / moved).exists():
            raise AssertionError(f"{moved} belongs in Test/, not at the root")

    sources = [
        CORE, SPEECH, TERMINAL, TERMINAL_SPEECH, VIBE_CODING, ANDROID
    ]
    sources += sorted(COMPILERS.glob("*.rs"))
    sources += sorted((COMPILERS / "Java").glob("*.rs"))
    sources += sorted((COMPILERS / "Kotlin").glob("*.rs"))
    sources += sorted(INFRASTRUCTURE.glob("*.rs"))
    carried = [
        str(path.relative_to(ROOT))
        for path in sources
        if "#[test]" in path.read_text(encoding="utf-8")
    ]
    if carried:
        raise AssertionError(
            "a test belongs in Builder_Test.rs, not beside the code it tests: "
            + ", ".join(carried)
        )
    return f"{len(required)} files and {len(compilers)} compilers in place"

def check_comments() -> str:
    """No source file in this project carries a comment.

    Reading a file the way its own compiler reads it, so that a `//` inside a
    string is a `//` inside a string, and the one `#!` that is not a comment
    stays. A file that grew one back is named.
    """
    watched: list[Path] = [
        CORE,
        SPEECH,
        TERMINAL,
        TERMINAL_SPEECH,
        VIBE_CODING,
        CORE_TESTS,
        KOTLIN,
        TEST_DRIVER,
        ROOT / "build.gradle.kts",
        ROOT / "settings.gradle.kts",
        ROOT / "Builder/build.gradle.kts",
        ROOT / "Builder/Source/Main/Native/Builder.cpp",
        ROOT / "Builder/Source/Main/Native/Builder.hpp",
    ]
    watched += sorted(COMPILERS.glob("*.rs"))
    watched += sorted((COMPILERS / "Java").glob("*.rs"))
    watched += sorted((COMPILERS / "Kotlin").glob("*.rs"))
    watched += sorted(INFRASTRUCTURE.glob("*.rs"))
    watched += [ANDROID]

    found: list[str] = []
    for path in watched:
        text = path.read_text(encoding="utf-8")
        for line, why in comments_in(text, path.suffix):
            found.append(f"{path.relative_to(ROOT)}:{line} {why}")
    if found:
        raise AssertionError(
            "these lines carry a comment:\n  " + "\n  ".join(found[:20])
        )
    return f"{len(watched)} files, and none of them explains itself"

def comments_in(text: str, suffix: str) -> list[tuple[int, str]]:
    """Where a file comments, reading strings the way the language does."""
    found: list[tuple[int, str]] = []
    at = 0
    size = len(text)
    line = 1
    braced = suffix in (".kt", ".kts", ".java", ".cpp", ".hpp")

    if suffix == ".py" and text.startswith("#!"):
        stop = text.find("\n")
        stop = size if stop < 0 else stop + 1
        at = stop
        line = 2

    while at < size:
        here = text[at]
        if here == "\n":
            line += 1
            at += 1
            continue

        if suffix == ".rs" and here == "r":
            probe = at + 1
            while probe < size and text[probe] == "#":
                probe += 1
            if probe < size and text[probe] == '"' and (
                at == 0 or not (text[at - 1].isalnum() or text[at - 1] == "_")
            ):
                close = '"' + "#" * (probe - at - 1)
                stop = text.find(close, probe + 1)
                stop = size if stop < 0 else stop + len(close)
                line += text.count("\n", at, stop)
                at = stop
                continue

        if (braced or suffix == ".py") and text.startswith('"""', at):
            stop = text.find('"""', at + 3)
            stop = size if stop < 0 else stop + 3
            line += text.count("\n", at, stop)
            at = stop
            continue

        if here in "\"'":
            probe = at + 1
            while probe < size:
                if text[probe] == "\\":
                    probe += 2
                    continue
                if text[probe] == here or text[probe] == "\n":
                    probe += 1
                    break
                probe += 1
            line += text.count("\n", at, probe)
            at = probe
            continue

        if suffix == ".py" and here == "#":
            found.append((line, "a # comment"))
            while at < size and text[at] != "\n":
                at += 1
            continue

        if suffix != ".py" and text.startswith("//", at):
            found.append((line, "a // comment"))
            while at < size and text[at] != "\n":
                at += 1
            continue

        if suffix != ".py" and text.startswith("/*", at):
            found.append((line, "a /* comment"))
            stop = text.find("*/", at + 2)
            stop = size if stop < 0 else stop + 2
            line += text.count("\n", at, stop)
            at = stop
            continue

        at += 1
    return found

def check_format() -> str:
    """The Core is formatted the way rustfmt writes it."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    run(["cargo", "fmt", "--check"])
    return "rustfmt agrees"

def check_lint() -> str:
    """Clippy finds nothing, with warnings treated as errors."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    run(["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"])
    return "clippy is clean"

def tool_present(finder, name: str) -> bool:
    try:
        return finder(name).is_file()
    except Skip:
        return False

def sign_a_copy(package: Path) -> Path:
    """A signed copy of an unsigned package, for the checks that need one.

    Continuous integration has signing secrets and a developer does not, so
    the package built here is unsigned and the one built there is signed. Every
    check that reads a signed package therefore ran only there -- which is how
    a stale claim about signatures reached fifteen red runs without one red
    check on anybody's machine.

    So the gap is closed by signing a copy with a key made on the spot. It is
    not the release key and it is never installed; it exists so that the checks
    which need a signature have one to read.
    """
    try:
        apksigner = build_tool("apksigner")
    except Skip:
        return package
    if shutil.which("keytool") is None:
        return package

    work = WORK / "signed"
    work.mkdir(parents=True, exist_ok=True)
    store = work / "throwaway.jks"
    copy = work / package.name.replace("-unsigned", "-locally-signed")
    if not store.is_file():
        run([
            "keytool", "-genkeypair", "-keystore", str(store),
            "-storepass", "apklocal", "-keypass", "apklocal", "-alias", "local",
            "-keyalg", "RSA", "-keysize", "2048", "-validity", "3650",
            "-dname", "CN=Apk Builder Local, O=Apk Builder, C=TR",
        ])
    shutil.copyfile(package, copy)
    run([
        str(apksigner), "sign",
        "--ks", str(store), "--ks-pass", "pass:apklocal",
        "--key-pass", "pass:apklocal", "--ks-key-alias", "local",
        "--min-sdk-version", "30", str(copy),
    ])
    return copy

def check_core() -> str:
    """The Core's own suite, demanding every conformance check the tools here allow."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")

    built = sorted(ROOT.glob("Builder/build/outputs/apk/*/*.apk"))
    release = [one for one in built if "/release/" in one.as_posix()]
    signed = [one for one in release if not one.name.endswith("-unsigned.apk")]
    under_test = signed or [sign_a_copy(one) for one in release] or built

    env = dict(CONFORMANCE)
    demanded = list(CONFORMANCE)
    for flag, present in (
        ("APK_BUILDER_REQUIRE_AXML_CONFORMANCE", tool_present(build_tool, "aapt2")),
        ("APK_BUILDER_REQUIRE_RSA_CONFORMANCE", shutil.which("openssl") is not None),
        ("APK_BUILDER_REQUIRE_DEX_CONFORMANCE",
         tool_present(build_tool, "dexdump") and bool(built)),
        ("APK_BUILDER_REQUIRE_CLASS_CONFORMANCE",
         shutil.which("javap") is not None and bool(built)),
        ("APK_BUILDER_REQUIRE_BUNDLETOOL",
         shutil.which("bundletool") is not None
         and bool(sorted(ROOT.glob("Builder/build/outputs/bundle/*/*.aab")))),
        ("APK_BUILDER_REQUIRE_SELF_BUILT_APK",
         tool_present(build_tool, "apksigner") and shutil.which("openssl") is not None),
        ("APK_BUILDER_REQUIRE_SELF_POLICY", bool(built)),
    ):
        if present and flag not in env:
            env[flag] = "1"
            demanded.append(flag)
    if under_test:
        env["APK_BUILDER_PACKAGE_UNDER_TEST"] = str(under_test[0])

    done = run(["cargo", "test", "--locked"], env=env)
    passed = sum(
        int(found.group(1))
        for found in re.finditer(r"test result: ok\. (\d+) passed", done.stdout)
    )
    if passed == 0:
        raise AssertionError("no test reported a result\n" + done.stdout[-2000:])
    return f"{passed} tests passed, demanding {len(demanded)} conformance checks"

def check_dependencies() -> str:
    """The Core carries no third-party dependency."""
    lock = (ROOT / "Cargo.lock").read_text(encoding="utf-8")
    packages = lock.count("[[package]]")
    if packages != 1:
        raise AssertionError(f"Cargo.lock holds {packages} packages, expected 1")
    return "one package, and it is this one"

def check_strings() -> str:
    """Every language carries every string the interface asks for."""
    def names(path: Path) -> list[str]:
        return sorted(re.findall(r'name="([^"]*)"', path.read_text(encoding="utf-8")))

    base = names(RESOURCES / "values/strings.xml")
    if not base:
        raise AssertionError("values/strings.xml holds no string")

    translations = sorted(RESOURCES.glob("values-*/strings.xml"))
    for path in translations:
        here = names(path)
        if here != base:
            short = path.parent.name
            extra = sorted(set(here) - set(base))
            absent = sorted(set(base) - set(here))
            raise AssertionError(
                f"{short} differs: missing {absent or 'nothing'}, extra {extra or 'nothing'}"
            )
    if len(translations) < 9:
        raise AssertionError(f"{len(translations)} translations, expected at least 9")

    used = set(
        re.findall(r"(?<!android\.)\bR\.string\.([a-z_0-9]+)", interface())
    )
    generated = {"apk_builder_expected_certificate"}
    fromXml = {"app_name"}
    unresolved = sorted(used - set(base) - generated)
    if unresolved:
        raise AssertionError(f"the interface asks for strings nobody declares: {unresolved}")
    unused = sorted(set(base) - used - fromXml)
    if unused:
        raise AssertionError(f"strings nothing uses: {unused}")

    loose: list[str] = []
    for path in [RESOURCES / "values/strings.xml"] + translations:
        for number, line in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
            body = re.search(r">(.*)</string>", line)
            if not body:
                continue
            text = body.group(1)
            for at, letter in enumerate(text):
                if letter == "'" and (at == 0 or text[at - 1] != "\\"):
                    loose.append(f"{path.parent.name}:{number}")
                    break
    if loose:
        raise AssertionError(
            "an apostrophe in a string resource has to be escaped, or aapt2 "
            "refuses the whole file: " + ", ".join(loose[:12])
        )
    return f"{len(base)} strings in {len(translations) + 1} languages, none spare"

def release_apk() -> Path:
    """The package this build produced, which is the only one it produces.

    A build with no signing key configured leaves a `-unsigned` package beside
    nothing else; a build with one leaves the signed package. Prefer the signed
    one where both are somehow present.
    """
    made = sorted(ROOT.glob("Builder/build/outputs/apk/release/*.apk"))
    if not made:
        raise Skip("the release package has not been built here")
    signed = [one for one in made if not one.name.endswith("-unsigned.apk")]
    return (signed or made)[0]

def check_release_apk() -> str:
    """The package and the bundle both build. This project ships one variant."""
    gradle(":Builder:assembleRelease", ":Builder:bundleRelease")
    made = sorted(ROOT.glob("Builder/build/outputs/apk/release/*.apk"))
    bundles = sorted(ROOT.glob("Builder/build/outputs/bundle/release/*.aab"))
    if not made or not bundles:
        raise AssertionError("the release package or the bundle is missing")
    if sorted(ROOT.glob("Builder/build/outputs/apk/debug/*.apk")):
        raise AssertionError("a debug package was built; this project ships one variant")
    return (f"{made[0].name}, {made[0].stat().st_size // 1024} KB, "
            f"and {bundles[0].name}")

def check_bridge() -> str:
    """Every native method Kotlin declares is exported by every ABI, and nothing else is."""
    apk = release_apk()
    nm = ndk_tool("llvm-nm")

    declared = sorted(
        f"Java_com_apk_builder_Builder_{name}"
        for name in re.findall(r"external fun (native[A-Za-z]*)", interface())
    ) + ["JNI_OnLoad"]
    declared = sorted(declared)

    work = WORK / "bridge"
    shutil.rmtree(work, ignore_errors=True)
    work.mkdir(parents=True)
    with zipfile.ZipFile(apk) as archive:
        archive.extractall(work)

    checked = 0
    for abi in ("arm64-v8a", "x86_64"):
        library = work / f"lib/{abi}/libapk_builder.so"
        if not library.is_file():
            raise AssertionError(f"no native library for {abi}")
        listed = run([str(nm), "-D", "--defined-only", str(library)])
        exported = sorted(
            line.split()[2] for line in listed.stdout.splitlines() if len(line.split()) >= 3
        )
        if exported != declared:
            raise AssertionError(
                f"{abi} exports {sorted(set(exported) - set(declared))} extra, "
                f"{sorted(set(declared) - set(exported))} missing"
            )
        if any("apk_builder_state_report" in name for name in exported):
            raise AssertionError(f"{abi} leaks the Core's own symbols")
        checked += 1
    return f"{len(declared)} symbols agree across {checked} ABIs"

def check_reproducible() -> str:
    """The package and the bundle are the same bytes across a clean rebuild.

    Both are rebuilt, not only the package: this check runs last, so whatever
    it leaves behind is what gets published, and a run that rebuilt only the
    package would quietly drop the bundle.
    """
    if os.environ.get("APK_BUILDER_SKIP_REPRODUCIBLE"):
        raise Skip("asked to skip the rebuild")
    import hashlib

    def digest(pattern: str, what: str) -> str:
        made = sorted(ROOT.glob(pattern))
        if not made:
            raise AssertionError(f"no {what} to weigh")
        return hashlib.sha256(made[0].read_bytes()).hexdigest()

    both = (":Builder:assembleRelease", ":Builder:bundleRelease")
    weighed = (
        ("Builder/build/outputs/apk/release/*.apk", "release package"),
        ("Builder/build/outputs/bundle/release/*.aab", "bundle"),
    )

    gradle(*both)
    first = [digest(pattern, what) for pattern, what in weighed]
    gradle("clean")
    run(["cargo", "clean"])
    gradle(*both)
    second = [digest(pattern, what) for pattern, what in weighed]

    for (_, what), before, after in zip(weighed, first, second):
        if before != after:
            raise AssertionError(f"the {what} is not reproducible\n{before}\n{after}")
    return f"package and bundle identical across a clean rebuild, {first[0][:16]}"

def check_installable() -> str:
    """The packages this build produces are ones Android would accept."""
    gradle(":Builder:verifyApkClasses", ":Builder:verifyApkInstallability", "--rerun-tasks")
    return "classes present and signatures accepted"

def check_interpolation() -> str:
    """No string the interface shows carries a name instead of the thing named.

    Kotlin writes a literal dollar as ${'$'}, which is the same six characters
    a template needs to be escaped with. Get it the wrong way round and the
    screen says ${it.packageName} where the package name should be, an import
    written into somebody's file reads `import ${qualified};`, and the intent
    that asks for install permission carries the URI `package:${packageName}`,
    which Android opens and closes again because no such package exists.

    Nothing in this interface wants a literal dollar in a template, so the rule
    is simply that the sequence may not appear. A dollar inside a character
    literal is a different thing and is left alone.
    """
    text = KOTLIN.read_text(encoding="utf-8")
    found = [
        index + 1
        for index, line in enumerate(text.split("\n"))
        if "${'$'}" in line
    ]
    if found:
        raise AssertionError(
            "these lines write a dollar where a value belongs: "
            + ", ".join(f"{KOTLIN.name}:{line}" for line in found[:20])
        )
    return f"no escaped template in {len(text.split(chr(10)))} lines of interface"

DIAGNOSTIC_MAKERS = ("fail", "problem", "reject", "diagnostic", "io_failure", "path_failure", "new")

def spoken_surface() -> tuple[set[str], set[str], list[str]]:
    """Every sentence and every context label a diagnostic can carry.

    A diagnostic is built with a code and a fixed sentence, and everything that
    varies goes into a context line written Label: value. That shape is what
    makes the whole surface translatable: the sentence is looked up whole, and
    the label is looked up on its own with the value left alone.
    """
    label = re.compile(r"^([^:{}]{1,40}): ")
    lead = re.compile(r"^([A-Za-z][A-Za-z]*(?: [a-z]+){0,3})[ ]\{")
    lines: set[str] = set()
    words: set[str] = set()
    woven: list[str] = []

    def plain(text: str) -> str:
        return re.sub(r"\\\n\s*", "", text).replace('\\"', '"').replace("\\'", "'").strip()

    for path in (
        [CORE, ANDROID]
        + sorted(COMPILERS.glob("*.rs"))
        + sorted((COMPILERS / "Java").glob("*.rs"))
        + sorted((COMPILERS / "Kotlin").glob("*.rs"))
        + sorted(INFRASTRUCTURE.glob("*.rs"))
    ):
        text = path.read_text(encoding="utf-8")
        text = text.split("\n#[cfg(test)]\n", 1)[0]
        for found in re.finditer(r'"(E(?:[A-Z]\d{3}|\d{4}))"', text):
            opened = text.rfind("(", max(0, found.start() - 400), found.start())
            if opened < 0:
                continue
            named = re.search(r"([A-Za-z_][A-Za-z_0-9:]*)\s*$", text[max(0, opened - 20):opened])
            if not named or named.group(1).split("::")[-1] not in DIAGNOSTIC_MAKERS:
                continue
            if text[opened + 1:found.start()].strip(" \n\t") not in ("", ","):
                continue
            rest = text[found.end():]
            at = 0
            while True:
                step = re.match(
                    r"\s*,\s*(?:[A-Za-z_][A-Za-z_0-9]*::)*[A-Za-z_][A-Za-z_0-9]*\s*(?=,)",
                    rest[at:],
                )
                if not step:
                    break
                at += step.end()
            while True:
                said = re.match(r'\s*,\s*(format!\s*\(\s*)?"((?:[^"\\]|\\.)*)"', rest[at:], re.S)
                if not said:
                    break
                if not said.group(1) and re.fullmatch(
                    r"[a-z][a-z0-9]*(?:\.[a-z][a-z0-9]*)+", said.group(2)
                ):
                    at += said.end()
                    continue
                if said.group(1):
                    woven.append(f"{path.name}: {found.group(1)} builds its sentence from parts")
                else:
                    lines.add(plain(said.group(2)))
                break

        if path == CORE:
            policy = text
            for rule in re.finditer(
                r'\(\s*"E[A-Z]\d{3}",\s*"[a-z-]+",\s*"((?:[^"\\]|\\.)*)",\s*\)', policy, re.S
            ):
                lines.add(plain(rule.group(1)))
            for said in re.finditer(
                r'(?:why|remedy):\s*"((?:[^"\\]|\\.)*)"', policy, re.S
            ):
                lines.add(plain(said.group(1)))
            for built in re.finditer(r'(?:why|remedy):\s*format!', policy):
                woven.append("the policy builds a finding from parts")
                del built

        for said in re.finditer(r'with_suggestion\(\s*(format!\(\s*)?"((?:[^"\\]|\\.)*)"', text, re.S):
            if said.group(1):
                woven.append(f"{path.name}: a suggestion is built from parts")
            else:
                lines.add(plain(said.group(2)))

        for said in re.finditer(
            r'with_context\(\s*(?:format!\(\s*)?"((?:[^"\\]|\\.)*)"', text, re.S
        ):
            for part in plain(said.group(1)).split(", "):
                head = label.match(part)
                if head:
                    if "{" not in head.group(1):
                        words.add(head.group(1))
                    continue
                head = lead.match(part)
                if head:
                    words.add(head.group(1))
                    continue
                if "{" not in part:
                    lines.add(part)

    for folder in ("Java", "Kotlin"):
        fixer = (COMPILERS / folder / f"{folder}_Fixer.rs").read_text(encoding="utf-8")
        opened = fixer.index("pub const REPAIR_TITLES: &[&str] = &[")
        closed = fixer.index("\n];", opened)
        for said in re.finditer(r'"((?:[^"\\]|\\.)*)"\s*,', fixer[opened + 36:closed]):
            lines.add(plain(said.group(1)))

    for folder, prefix in (("Java", "EJ"), ("Kotlin", "EK"), ("Compiler_Infrastructure", "EI")):
        catalogue = (COMPILERS / folder / f"{folder}_Diagnostic.rs").read_text(encoding="utf-8")
        opened = catalogue.index("pub const CATALOGUE: &[(&str, &str)] = &[")
        closed = catalogue.index("\n];", opened)
        for said in re.finditer(
            r'\(\s*"' + prefix + r'\d{3}"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)\s*,',
            catalogue[opened:closed],
            re.S,
        ):
            lines.add(plain(said.group(1)))

    return lines, words, woven

def spoken_table(name: str) -> dict[str, str]:
    """One of the tables the speech module carries, read back out of it."""
    text = SPEECH.read_text(encoding="utf-8")
    opened = text.index(f"const {name}: &[(&str, &str)] = &[")
    closed = text.index("\n];", opened)
    body = text[text.index("\n", opened):closed]
    pair = re.compile(r'\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)\s*,')
    said: dict[str, str] = {}
    at = 0
    for found in pair.finditer(body):
        loose = body[at:found.start()].strip()
        if loose:
            raise AssertionError(f"{name} holds something that is not a pair: {loose[:60]}")
        english = found.group(1).replace('\\"', '"').replace("\\\\", "\\")
        if english in said:
            raise AssertionError(f"{name} holds {english!r} twice")
        said[english] = found.group(2)
        at = found.end()
    loose = body[at:].strip()
    if loose:
        raise AssertionError(f"{name} ends in something that is not a pair: {loose[:60]}")
    if list(said) != sorted(said):
        raise AssertionError(f"{name} is not in order, so a lookup would miss")
    return said

def check_speech() -> str:
    """Every refusal this build can make is written in the languages it speaks.

    Core knows nothing about Android resources, so the sentences live here with
    the codes that raise them. This check is what keeps them honest: a new
    diagnostic without a translation fails the gate by name, rather than
    reaching a phone in English.
    """
    lines, words, woven = spoken_surface()
    if woven:
        raise AssertionError(
            "a message assembled by format! cannot be looked up: " + "; ".join(woven[:8])
        )
    trouble = []
    spoken = [
        (name, wanted)
        for language in (
            "TURKISH",
            "GERMAN",
            "SPANISH",
            "FRENCH",
            "ITALIAN",
            "PORTUGUESE",
            "RUSSIAN",
            "ARABIC",
            "CHINESE",
            "JAPANESE",
            "KOREAN",
            "HINDI",
            "INDONESIAN",
            "BRAZILIAN",
        )
        for name, wanted in ((f"{language}_LINES", lines), (f"{language}_WORDS", words))
    ]
    for name, wanted in spoken:
        said = spoken_table(name)
        missing = sorted(wanted - set(said))
        spare = sorted(set(said) - wanted)
        if missing:
            trouble.append(f"{name} is missing {len(missing)}: " + "; ".join(
                repr(one[:70]) for one in missing[:6]
            ))
        if spare:
            trouble.append(f"{name} holds {len(spare)} nothing says: " + "; ".join(
                repr(one[:70]) for one in spare[:6]
            ))
    if trouble:
        raise AssertionError(" | ".join(trouble))
    return (
        f"{len(lines)} sentences and {len(words)} labels "
        f"in each of {len(spoken) // 2} languages spoken"
    )

def check_independence() -> str:
    """The compiler infrastructure stands on its own.

    Every front end lowers into it and every back end reads out of it, so it
    may not reach back into any of them, nor into the Android build, the
    terminal or the assistant. It may use the diagnostic type, the failure
    classes and the standard library, and nothing else; and nothing in it may
    iterate in an order that depends on hashing, or the same module could be
    written two different ways.
    """
    allowed = {"diag", "FailureClass"}
    found: list[str] = []
    reached: set[str] = set()
    for path in sorted(INFRASTRUCTURE.glob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for match in re.finditer(r"\bcrate::([A-Za-z_][A-Za-z_0-9]*)", text):
            reached.add(match.group(1))
            if path.name == "Compiler_Infrastructure.rs" and match.group(1) not in allowed:
                line = text.count("\n", 0, match.start()) + 1
                found.append(f"{path.relative_to(ROOT)}:{line} reaches crate::{match.group(1)}")
        for match in re.finditer(r"\b(HashMap|HashSet|RandomState|extern crate|use std::env)\b", text):
            line = text.count("\n", 0, match.start()) + 1
            found.append(f"{path.relative_to(ROOT)}:{line} uses {match.group(1)}")
    catalogue = (INFRASTRUCTURE / "Compiler_Infrastructure_Diagnostic.rs").read_text(encoding="utf-8")
    for match in re.finditer(r"\bcrate::([A-Za-z_][A-Za-z_0-9]*)", catalogue):
        if match.group(1) != "speech":
            found.append(f"the catalogue reaches crate::{match.group(1)}")
    if found:
        raise AssertionError("the infrastructure is not independent:\n  " + "\n  ".join(found[:20]))
    return f"it reaches only {', '.join(sorted(reached))}"

SUITE_LINE = re.compile(
    r"SUITE (\S+) passed=(\d+) failed=(\d+) out_of_scope=(\d+) total=(\d+) digest=([0-9a-f]+)$",
    re.M,
)

def bundles_present() -> bool:
    """The reference bundles are in this repository's history."""
    done = run(["git", "ls-tree", "HEAD", "--", "New"], check=False)
    return done.returncode == 0 and "New" in done.stdout

def run_suites(names: list[str], threads: int | None = None,
               module: str = "reference_checks") -> dict[str, dict]:
    """Every named suite, run once, read back from the lines it prints."""
    env = {"APK_BUILDER_SUITE_THREADS": str(threads)} if threads else {}
    tests = [
        f"tests::{SUITE_HOMES.get(name, module)}::suite_{name.replace('-', '_')}"
        for name in names
    ]
    done = run(
        ["cargo", "test", "--locked", "--lib", "--", "--ignored", "--exact", *tests,
         "--nocapture", "--test-threads=1"],
        env=env,
    )
    found: dict[str, dict] = {}
    for match in SUITE_LINE.finditer(done.stdout):
        found[match.group(1)] = {
            "passed": int(match.group(2)),
            "failed": int(match.group(3)),
            "out_of_scope": int(match.group(4)),
            "total": int(match.group(5)),
            "digest": match.group(6),
        }
    silent = [name for name in names if name not in found]
    if silent:
        raise AssertionError(f"these suites printed no result: {silent}")
    return found

def hold_ground(found: dict[str, dict]) -> str:
    """No suite may give back a test it once passed.

    The baseline holds, for every suite, how many tests passed and how many
    failed the last time someone recorded it. A run that passes fewer or fails
    more is a regression and fails the gate by name. A run that does better is
    reported, so the baseline can be raised with --record and the gain kept.
    """
    held = json.loads(BASELINE.read_text(encoding="utf-8"))
    suites = held.setdefault("suites", {})
    if RECORDING["on"]:
        for name, now in found.items():
            suites[name] = {key: now[key] for key in ("passed", "failed", "out_of_scope", "total")}
        BASELINE.write_text(json.dumps(held, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return "recorded " + ", ".join(f"{name} {now['passed']}/{now['total']}" for name, now in found.items())
    trouble: list[str] = []
    gains: list[str] = []
    for name, now in found.items():
        was = suites.get(name)
        if was is None:
            trouble.append(f"{name} has no baseline; run with --record")
            continue
        if now["passed"] < was["passed"]:
            trouble.append(f"{name} passes {now['passed']}, the baseline says {was['passed']}")
        if now["failed"] > was["failed"]:
            trouble.append(f"{name} fails {now['failed']}, the baseline allows {was['failed']}")
        if now["passed"] > was["passed"] or now["failed"] < was["failed"]:
            gains.append(name)
    if trouble:
        raise AssertionError("; ".join(trouble))
    summary = ", ".join(
        f"{name} {now['passed']}/{now['total']}"
        + (f" ({now['out_of_scope']} out of scope)" if now["out_of_scope"] else "")
        for name, now in found.items()
    )
    if gains:
        summary += f"; better than the baseline in {', '.join(gains)}, record it"
    return summary

def check_suites() -> str:
    """Every reference suite keeps every test it has ever passed."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    return hold_ground(run_suites(REFERENCE_SUITES))

def check_differential() -> str:
    """Our engines and an independent implementation agree on the same input."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if shutil.which("grep") is None or shutil.which("timeout") is None:
        raise Skip("grep or timeout is not on this machine")
    if shutil.which("llvm-tblgen") is None:
        raise Skip("llvm-tblgen is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    return hold_ground(run_suites(ORACLE_SUITES))

def check_linker() -> str:
    """Our ld.lld and lld-link keep every LLD lit test they have ever passed."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    missing = [tool for tool in ("llvm-mc", "yaml2obj", "llvm-readobj", "llvm-objdump")
               if shutil.which(tool) is None]
    if missing:
        raise Skip(f"the LLVM tools the LLD tests run are not here: {', '.join(missing)}")
    return hold_ground(run_suites(LINKER_SUITES, module="linker_lit"))

def check_assembler() -> str:
    """Our x86 assembler keeps every LLVM MC test it has ever passed."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    return hold_ground(run_suites(ASSEMBLER_SUITES, module="x86_assembly"))

def kotlin_standard_library() -> Path | None:
    """The newest kotlin-stdlib this machine holds, the way the Core suite finds it."""
    named = os.environ.get("APK_BUILDER_KOTLIN_STDLIB")
    if named:
        return Path(named) if Path(named).is_file() else None
    base = Path.home() / ".gradle/caches/modules-2/files-2.1/org.jetbrains.kotlin/kotlin-stdlib"
    found = sorted(base.glob("*/*/kotlin-stdlib-*.jar")) if base.is_dir() else []
    return found[-1] if found else None

def check_kotlin_omni() -> str:
    """Every Kotlin box test the JVM answers OK answers OK on Omni IR too, and natively as interpreted."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    if shutil.which("java") is None and not os.environ.get("JAVA_HOME"):
        raise Skip("java is not on this machine")
    if kotlin_standard_library() is None:
        raise Skip("no kotlin-stdlib is on this machine")
    return hold_ground(run_suites(KOTLIN_SUITES))

def check_determinism() -> str:
    """One worker or many, every suite reaches the same verdict on every test."""
    if shutil.which("cargo") is None:
        raise Skip("cargo is not on this machine")
    if not bundles_present():
        raise Skip("the reference bundles are not in this checkout")
    names = REFERENCE_SUITES + ORACLE_SUITES
    alone = run_suites(names, threads=1)
    together = run_suites(names, threads=max(2, os.cpu_count() or 2))
    differ = [name for name in names if alone[name]["digest"] != together[name]["digest"]]
    if differ:
        raise AssertionError(f"these suites answer differently with more workers: {differ}")
    return f"{len(names)} suites, identical digests on one worker and on many"

CHECKS: dict[str, tuple[str, object]] = {
    "layout": ("the tree is the agreed one", check_layout),
    "comments": ("no file explains itself", check_comments),
    "templates": ("no name shown where a value belongs", check_interpolation),
    "format": ("rustfmt", check_format),
    "lint": ("clippy", check_lint),
    "core": ("the Core suite", check_core),
    "dependencies": ("no third-party dependency", check_dependencies),
    "strings": ("every language, every string", check_strings),
    "speech": ("every refusal in the reader's language", check_speech),
    "independence": ("the compiler infrastructure stands alone", check_independence),
    "release": ("the package and the bundle", check_release_apk),
    "bridge": ("the native bridge", check_bridge),
    "installable": ("what Android would accept", check_installable),
    "reproducible": ("byte-for-byte rebuild", check_reproducible),
    "suites": ("every reference suite holds its ground", check_suites),
    "differential": ("our engines agree with independent ones", check_differential),
    "determinism": ("the same verdicts on one worker and many", check_determinism),
    "kotlin-omni": ("Kotlin answers the same on Omni IR as on the JVM", check_kotlin_omni),
    "linker": ("our linkers hold their ground on the LLD tests", check_linker),
    "assembler": ("our x86 assembler holds its ground on the LLVM MC tests", check_assembler),
}

DEFAULT = [
    "layout", "comments", "templates", "format", "lint", "dependencies", "strings", "speech",
    "independence",
    "release", "core", "bridge", "installable",
]

def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("checks", nargs="*", help="checks to run, or none for the usual set")
    parser.add_argument("--list", action="store_true", help="name every check and stop")
    parser.add_argument("--all", action="store_true", help="every check, the slow ones too")
    parser.add_argument("--strict", action="store_true",
                        help="a check that cannot run here is a failure")
    parser.add_argument("--record", action="store_true",
                        help="write the suite results as the new baseline")
    chosen = parser.parse_args()
    RECORDING["on"] = chosen.record

    if chosen.list:
        width = max(len(name) for name in CHECKS)
        for name, (what, _) in CHECKS.items():
            mark = " " if name in DEFAULT else "*"
            print(f"  {mark} {name:<{width}}  {what}")
        print("\n  * not in the usual set; ask for it by name or pass --all")
        return 0

    if chosen.checks:
        unknown = [name for name in chosen.checks if name not in CHECKS]
        if unknown:
            print(f"no such check: {', '.join(unknown)}", file=sys.stderr)
            return 2
        wanted = chosen.checks
    elif chosen.all:
        wanted = list(CHECKS)
    else:
        wanted = DEFAULT

    WORK.mkdir(exist_ok=True)
    (WORK / ".gitignore").write_text("*\n", encoding="utf-8")

    results: list[Result] = []
    width = max(len(name) for name in wanted)
    for name in wanted:
        what, check = CHECKS[name]
        print(f"  {name:<{width}}  ...", end="", flush=True)
        started = time.monotonic()
        try:
            detail = check()
            state = "ok"
        except Skip as why:
            detail, state = str(why), "skipped"
        except AssertionError as why:
            detail, state = str(why), "failed"
        except Exception as why:
            detail, state = f"{type(why).__name__}: {why}", "failed"
        spent = time.monotonic() - started
        results.append(Result(name, state, spent, detail))
        mark = {"ok": "ok", "skipped": "--", "failed": "FAILED"}[state]
        first = detail.splitlines()[0] if detail else ""
        print(f"\r  {name:<{width}}  {mark:<6} {spent:6.1f}s  {first}")

    failed = [one for one in results if one.state == "failed"]
    skipped = [one for one in results if one.state == "skipped"]

    print()
    for one in failed:
        print(f"{one.name} failed:")
        for line in one.detail.splitlines():
            print(f"    {line}")
        print()

    summary = (
        f"{len(results) - len(failed) - len(skipped)} passed, "
        f"{len(failed)} failed, {len(skipped)} skipped, "
        f"{sum(one.seconds for one in results):.0f}s"
    )
    if failed:
        print(summary)
        return 1
    if skipped and chosen.strict:
        print(f"{summary} — and --strict makes a skipped check a failure")
        return 1
    print(summary)
    return 0

if __name__ == "__main__":
    sys.exit(main())
