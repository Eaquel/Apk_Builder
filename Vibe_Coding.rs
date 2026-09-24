use crate::json::{self, Json, Writer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Keyless,
    Free,
    Own,
}

impl Tier {
    pub fn name(self) -> &'static str {
        match self {
            Tier::Keyless => "keyless",
            Tier::Free => "free",
            Tier::Own => "byok",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Open,
    Anthropic,
    Gemini,
    Dvm,
    Raw,
}

impl Format {
    pub const CHOSEN: [Format; 4] = [Format::Open, Format::Anthropic, Format::Gemini, Format::Raw];

    pub fn name(self) -> &'static str {
        match self {
            Format::Open => "openai",
            Format::Anthropic => "anthropic",
            Format::Gemini => "gemini",
            Format::Dvm => "dvm",
            Format::Raw => "raw",
        }
    }

    pub fn parse(said: &str) -> Option<Format> {
        [
            Format::Open,
            Format::Anthropic,
            Format::Gemini,
            Format::Dvm,
            Format::Raw,
        ]
        .into_iter()
        .find(|one| one.name().eq_ignore_ascii_case(said.trim()))
    }

    pub fn sign(self) -> Sign {
        match self {
            Format::Open | Format::Raw => Sign::Bearer,
            Format::Anthropic => Sign::Anthropic,
            Format::Gemini => Sign::Google,
            Format::Dvm => Sign::Lightning,
        }
    }

    pub fn probe(self) -> (&'static str, &'static str) {
        match self {
            Format::Open | Format::Gemini => ("GET", "/models"),
            Format::Anthropic => ("GET", "/v1/models"),
            Format::Dvm | Format::Raw => ("POST", ""),
        }
    }

    pub fn wire(self) -> Wire {
        match self {
            Format::Open => Wire::Reasoning("high"),
            Format::Anthropic => Wire::Adaptive,
            Format::Gemini => Wire::Budget(FLASH_BUDGETS),
            Format::Dvm | Format::Raw => Wire::Prompted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Nothing,
    Bearer,
    Anthropic,
    Google,
    GitHub,
    Lightning,
}

impl Sign {
    pub fn name(self) -> &'static str {
        match self {
            Sign::Nothing => "none",
            Sign::Bearer => "bearer",
            Sign::Anthropic => "x-api-key",
            Sign::Google => "x-goog-api-key",
            Sign::GitHub => "github",
            Sign::Lightning => "l402",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Privacy {
    Untrained,
    Trained,
    Unverified,
    Contract,
}

impl Privacy {
    pub fn name(self) -> &'static str {
        match self {
            Privacy::Untrained => "untrained",
            Privacy::Trained => "trained",
            Privacy::Unverified => "unverified",
            Privacy::Contract => "contract",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Effort {
    Low,
    Medium,
    High,
    Deep,
}

impl Effort {
    pub const ALL: [Effort; 4] = [Effort::Low, Effort::Medium, Effort::High, Effort::Deep];

    pub fn name(self) -> &'static str {
        match self {
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::Deep => "deep",
        }
    }

    pub fn parse(said: &str) -> Option<Effort> {
        match said.trim().to_ascii_lowercase().as_str() {
            "low" => Some(Effort::Low),
            "medium" | "med" => Some(Effort::Medium),
            "high" => Some(Effort::High),
            "deep" | "max" | "xhigh" => Some(Effort::Deep),
            _ => None,
        }
    }

    fn rank(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wire {
    Adaptive,
    Budget([u64; 4]),
    Reasoning(&'static str),
    Prompted,
    Output([u64; 4]),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    pub level: Effort,
    pub param: &'static str,
    pub value: String,
    pub most_tokens: u64,
    pub hint: &'static str,
}

pub const MOST_TOKENS: u64 = 64_000;

pub const LEAST_TOKENS: u64 = 256;

const FLASH_BUDGETS: [u64; 4] = [1_024, 8_192, 24_576, 24_576];

const LITE_BUDGETS: [u64; 4] = [512, 4_096, 16_384, 24_576];

const PRO_BUDGETS: [u64; 4] = [1_024, 8_192, 24_576, 32_768];

const HAIKU_BUDGETS: [u64; 4] = [2_048, 8_192, 16_384, 32_000];

const ADAPTIVE: [&str; 4] = ["low", "medium", "high", "max"];

const REASONING_TOKENS: [u64; 4] = [4_096, 16_384, 32_768, MOST_TOKENS];

const PROMPTED_TOKENS: [u64; 4] = [2_048, 4_096, 8_192, 16_384];

const HINTS: [&str; 4] = [
    "Answer directly and briefly.",
    "Think the problem through before you answer.",
    "Reason step by step, check your work, and answer thoroughly.",
    "Reason exhaustively: weigh every alternative, verify each step, then answer completely.",
];

pub fn setting(wire: Wire, level: Effort) -> Setting {
    let at = level.rank();
    let (param, value, most_tokens, hint) = match wire {
        Wire::Adaptive => ("effort", ADAPTIVE[at].to_string(), MOST_TOKENS, ""),
        Wire::Budget(budgets) => (
            "budget",
            budgets[at].to_string(),
            (budgets[at] + 16_384).min(MOST_TOKENS),
            "",
        ),
        Wire::Reasoning(deep) => {
            let value = if level == Effort::Deep {
                deep
            } else {
                level.name()
            };
            (
                "reasoning_effort",
                value.to_string(),
                REASONING_TOKENS[at],
                "",
            )
        }
        Wire::Prompted => (
            "hint",
            level.name().to_string(),
            PROMPTED_TOKENS[at],
            HINTS[at],
        ),
        Wire::Output(caps) => ("max_tokens", caps[at].to_string(), caps[at], ""),
    };
    Setting {
        level,
        param,
        value,
        most_tokens,
        hint,
    }
}

pub fn settle(efforts: &[Effort], wanted: Effort) -> Option<Effort> {
    if efforts.contains(&wanted) {
        return Some(wanted);
    }
    efforts
        .iter()
        .copied()
        .min_by_key(|one| (one.rank().abs_diff(wanted.rank()), usize::MAX - one.rank()))
}

pub fn room(window: u64, need: u64, most: u64) -> u64 {
    window.saturating_sub(need).min(most).max(LEAST_TOKENS)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Quota {
    pub rpm: u64,
    pub rpd: u64,
    pub tpm: u64,
    pub tpd: u64,
}

const OPEN: Quota = Quota {
    rpm: 0,
    rpd: 0,
    tpm: 0,
    tpd: 0,
};

#[derive(Debug)]
pub struct Model {
    pub id: &'static str,
    pub shown: &'static str,
    pub window: u64,
    pub efforts: &'static [Effort],
    pub wire: Wire,
}

#[derive(Debug)]
pub struct Provider {
    pub id: &'static str,
    pub shown: &'static str,
    pub tier: Tier,
    pub format: Format,
    pub sign: Sign,
    pub base: &'static str,
    pub home: &'static str,
    pub privacy: Privacy,
    pub notice: &'static str,
    pub quota: Quota,
    pub free_window: u64,
    pub last_resort: bool,
    pub models: &'static [Model],
}

impl Provider {
    pub fn model(&self, id: &str) -> Option<&'static Model> {
        self.models.iter().find(|one| one.id == id)
    }

    pub fn window(&self, model: &Model) -> u64 {
        if self.free_window > 0 {
            self.free_window.min(model.window)
        } else {
            model.window
        }
    }
}

const LOW_HIGH: &[Effort] = &[Effort::Low, Effort::High];

const LOW_TO_HIGH: &[Effort] = &[Effort::Low, Effort::Medium, Effort::High];

const LOW_TO_DEEP: &[Effort] = &[Effort::Low, Effort::Medium, Effort::High, Effort::Deep];

const HIGH_DEEP: &[Effort] = &[Effort::High, Effort::Deep];

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "groq",
        shown: "Groq",
        tier: Tier::Free,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.groq.com/openai/v1",
        home: "console.groq.com",
        privacy: Privacy::Untrained,
        notice: "groq",
        quota: Quota {
            rpm: 30,
            rpd: 14_400,
            tpm: 6_000,
            tpd: 500_000,
        },
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "llama-3.3-70b-versatile",
            shown: "Llama 3.3 70B",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "cerebras",
        shown: "Cerebras",
        tier: Tier::Free,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.cerebras.ai/v1",
        home: "cloud.cerebras.ai",
        privacy: Privacy::Untrained,
        notice: "untrained",
        quota: Quota {
            rpm: 5,
            rpd: 0,
            tpm: 0,
            tpd: 1_000_000,
        },
        free_window: 8_192,
        last_resort: false,
        models: &[Model {
            id: "gpt-oss-120b",
            shown: "GPT-OSS 120B",
            window: 131_072,
            efforts: LOW_TO_HIGH,
            wire: Wire::Reasoning("high"),
        }],
    },
    Provider {
        id: "sambanova",
        shown: "SambaNova",
        tier: Tier::Free,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.sambanova.ai/v1",
        home: "cloud.sambanova.ai",
        privacy: Privacy::Untrained,
        notice: "untrained",
        quota: Quota {
            rpm: 20,
            rpd: 0,
            tpm: 0,
            tpd: 200_000,
        },
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "Meta-Llama-3.3-70B-Instruct",
            shown: "Llama 3.3 70B",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "google",
        shown: "Google Gemini",
        tier: Tier::Free,
        format: Format::Gemini,
        sign: Sign::Google,
        base: "https://generativelanguage.googleapis.com/v1beta",
        home: "aistudio.google.com",
        privacy: Privacy::Trained,
        notice: "gemini",
        quota: Quota {
            rpm: 15,
            rpd: 1_500,
            tpm: 0,
            tpd: 0,
        },
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "gemini-2.5-flash",
                shown: "Gemini 2.5 Flash",
                window: 1_048_576,
                efforts: LOW_TO_HIGH,
                wire: Wire::Budget(FLASH_BUDGETS),
            },
            Model {
                id: "gemini-2.5-flash-lite",
                shown: "Gemini 2.5 Flash-Lite",
                window: 1_048_576,
                efforts: LOW_TO_HIGH,
                wire: Wire::Budget(LITE_BUDGETS),
            },
            Model {
                id: "gemini-2.5-pro",
                shown: "Gemini 2.5 Pro",
                window: 1_048_576,
                efforts: LOW_TO_DEEP,
                wire: Wire::Budget(PRO_BUDGETS),
            },
        ],
    },
    Provider {
        id: "openrouter",
        shown: "OpenRouter",
        tier: Tier::Free,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://openrouter.ai/api/v1",
        home: "openrouter.ai",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: Quota {
            rpm: 20,
            rpd: 50,
            tpm: 0,
            tpd: 0,
        },
        free_window: 0,
        last_resort: true,
        models: &[Model {
            id: "meta-llama/llama-3.3-70b-instruct:free",
            shown: "Llama 3.3 70B",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "kilo",
        shown: "Kilo Gateway",
        tier: Tier::Keyless,
        format: Format::Open,
        sign: Sign::Nothing,
        base: "https://api.kilo.ai/api/gateway",
        home: "kilo.ai",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "kilo-auto/free",
            shown: "Kilo Auto",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "llm7",
        shown: "LLM7.io",
        tier: Tier::Keyless,
        format: Format::Open,
        sign: Sign::Nothing,
        base: "https://api.llm7.io/v1",
        home: "llm7.io",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "default",
                shown: "LLM7 Default",
                window: 131_072,
                efforts: LOW_HIGH,
                wire: Wire::Prompted,
            },
            Model {
                id: "GLM-5.3-Flash",
                shown: "GLM-5.3 Flash",
                window: 131_072,
                efforts: LOW_HIGH,
                wire: Wire::Prompted,
            },
        ],
    },
    Provider {
        id: "pollinations",
        shown: "Pollinations AI",
        tier: Tier::Keyless,
        format: Format::Open,
        sign: Sign::Nothing,
        base: "https://text.pollinations.ai/openai",
        home: "pollinations.ai",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "openai-fast",
            shown: "GPT-OSS 20B",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "keylessai",
        shown: "KeylessAI",
        tier: Tier::Keyless,
        format: Format::Open,
        sign: Sign::Nothing,
        base: "https://keylessai.thryx.workers.dev/v1",
        home: "keylessai.com",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "openai",
            shown: "KeylessAI",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "copilot",
        shown: "GitHub Copilot",
        tier: Tier::Free,
        format: Format::Open,
        sign: Sign::GitHub,
        base: "https://api.githubcopilot.com",
        home: "github.com/features/copilot",
        privacy: Privacy::Contract,
        notice: "",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "gpt-4.1",
                shown: "GPT-4.1",
                window: 128_000,
                efforts: LOW_HIGH,
                wire: Wire::Prompted,
            },
            Model {
                id: "gpt-5-mini",
                shown: "GPT-5 mini",
                window: 128_000,
                efforts: LOW_TO_HIGH,
                wire: Wire::Reasoning("medium"),
            },
            Model {
                id: "claude-sonnet-5",
                shown: "Claude Sonnet 5",
                window: 200_000,
                efforts: LOW_TO_HIGH,
                wire: Wire::Prompted,
            },
            Model {
                id: "gpt-5.2",
                shown: "GPT-5.2",
                window: 200_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Reasoning("high"),
            },
            Model {
                id: "gemini-2.5-pro",
                shown: "Gemini 2.5 Pro",
                window: 128_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Prompted,
            },
        ],
    },
    Provider {
        id: "anthropic",
        shown: "Anthropic",
        tier: Tier::Own,
        format: Format::Anthropic,
        sign: Sign::Anthropic,
        base: "https://api.anthropic.com",
        home: "console.anthropic.com",
        privacy: Privacy::Contract,
        notice: "",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "claude-opus-5-5",
                shown: "Claude Opus 5.5",
                window: 1_000_000,
                efforts: HIGH_DEEP,
                wire: Wire::Adaptive,
            },
            Model {
                id: "claude-opus-5",
                shown: "Claude Opus 5",
                window: 1_000_000,
                efforts: HIGH_DEEP,
                wire: Wire::Adaptive,
            },
            Model {
                id: "claude-sonnet-5",
                shown: "Claude Sonnet 5",
                window: 1_000_000,
                efforts: LOW_TO_HIGH,
                wire: Wire::Adaptive,
            },
            Model {
                id: "claude-fable-5-1",
                shown: "Claude Fable 5.1",
                window: 1_000_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Adaptive,
            },
            Model {
                id: "claude-haiku-4-5",
                shown: "Claude Haiku 4.5",
                window: 200_000,
                efforts: LOW_TO_HIGH,
                wire: Wire::Budget(HAIKU_BUDGETS),
            },
        ],
    },
    Provider {
        id: "openai",
        shown: "OpenAI",
        tier: Tier::Own,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.openai.com/v1",
        home: "platform.openai.com",
        privacy: Privacy::Contract,
        notice: "",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "gpt-5.2",
                shown: "GPT-5.2",
                window: 400_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Reasoning("xhigh"),
            },
            Model {
                id: "gpt-5.1",
                shown: "GPT-5.1",
                window: 400_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Reasoning("high"),
            },
            Model {
                id: "gpt-5-mini",
                shown: "GPT-5 mini",
                window: 400_000,
                efforts: LOW_TO_DEEP,
                wire: Wire::Reasoning("high"),
            },
        ],
    },
    Provider {
        id: "deepseek",
        shown: "DeepSeek",
        tier: Tier::Own,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.deepseek.com/v1",
        home: "platform.deepseek.com",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[
            Model {
                id: "deepseek-v4-pro",
                shown: "DeepSeek V4 Pro",
                window: 131_072,
                efforts: HIGH_DEEP,
                wire: Wire::Output([8_192, 16_384, 32_768, 65_536]),
            },
            Model {
                id: "deepseek-v4.1-flash",
                shown: "DeepSeek V4.1-Flash",
                window: 131_072,
                efforts: LOW_TO_HIGH,
                wire: Wire::Output([4_096, 8_192, 16_384, 32_768]),
            },
        ],
    },
    Provider {
        id: "openzoo",
        shown: "OpenZoo",
        tier: Tier::Own,
        format: Format::Open,
        sign: Sign::Bearer,
        base: "https://api.openzoo.fun/v1",
        home: "openzoo.ai",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "openzoo/auto",
            shown: "OpenZoo Auto",
            window: 131_072,
            efforts: LOW_HIGH,
            wire: Wire::Prompted,
        }],
    },
    Provider {
        id: "maximumsats",
        shown: "Maximum Sats DVM",
        tier: Tier::Own,
        format: Format::Dvm,
        sign: Sign::Lightning,
        base: "https://maximumsats.com/api/dvm",
        home: "maximumsats.com",
        privacy: Privacy::Unverified,
        notice: "unverified",
        quota: OPEN,
        free_window: 0,
        last_resort: false,
        models: &[Model {
            id: "dvm",
            shown: "Maximum Sats DVM",
            window: 32_768,
            efforts: &[],
            wire: Wire::Prompted,
        }],
    },
];

pub fn provider(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|one| one.id == id)
}

#[derive(Debug)]
pub struct Barred {
    pub name: &'static str,
    pub hosts: &'static [&'static str],
    pub why: &'static str,
}

pub const BARRED: &[Barred] = &[
    Barred {
        name: "NVIDIA NIM",
        hosts: &["nvidia.com"],
        why: "phone",
    },
    Barred {
        name: "Mistral AI",
        hosts: &["mistral.ai"],
        why: "phone",
    },
    Barred {
        name: "xAI",
        hosts: &["x.ai", "grok.com"],
        why: "training",
    },
    Barred {
        name: "Together AI",
        hosts: &["together.xyz", "together.ai"],
        why: "paid",
    },
    Barred {
        name: "Fireworks AI",
        hosts: &["fireworks.ai"],
        why: "paid",
    },
];

const LOCAL_HOSTS: &[&str] = &["localhost", "127.0.0.1", "10.0.2.2", "[::1]"];

pub fn host_of(url: &str) -> Option<(String, String)> {
    let (scheme, rest) = url.trim().split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    let authority = rest.split(['/', '?', '#']).next()?;
    let authority = authority.rsplit('@').next()?;
    let host = if authority.starts_with('[') {
        authority
            .split_inclusive(']')
            .next()
            .unwrap_or(authority)
            .to_string()
    } else {
        authority.split(':').next().unwrap_or(authority).to_string()
    };
    let host = host.to_ascii_lowercase();
    if host.is_empty() || host.chars().any(|one| one.is_whitespace()) {
        return None;
    }
    Some((scheme, host))
}

pub fn barred(url: &str) -> Option<&'static Barred> {
    let (_, host) = host_of(url)?;
    BARRED.iter().find(|one| {
        one.hosts
            .iter()
            .any(|known| host == *known || host.ends_with(&format!(".{known}")))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    Simple,
    Medium,
    Complex,
}

impl Task {
    pub fn name(self) -> &'static str {
        match self {
            Task::Simple => "simple",
            Task::Medium => "medium",
            Task::Complex => "complex",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ask {
    pub tokens: u64,
    pub files: u64,
    pub code: bool,
    pub failure: bool,
}

pub const SIMPLE_TOKENS: u64 = 600;

pub const WIDE_TOKENS: u64 = 24_000;

pub const CODE_TOKENS: u64 = 12_000;

pub const FAILURE_TOKENS: u64 = 4_000;

pub const MANY_FILES: u64 = 3;

pub fn classify(ask: &Ask) -> Task {
    let complex = ask.tokens > WIDE_TOKENS
        || ask.files >= MANY_FILES
        || (ask.code && ask.tokens > CODE_TOKENS)
        || (ask.failure && ask.tokens > FAILURE_TOKENS);
    if complex {
        return Task::Complex;
    }
    if ask.tokens < SIMPLE_TOKENS && ask.files == 0 && !ask.code && !ask.failure {
        return Task::Simple;
    }
    Task::Medium
}

const FREE_TAIL: &[(&str, &str)] = &[
    ("kilo", "kilo-auto/free"),
    ("llm7", "default"),
    ("pollinations", "openai-fast"),
    ("keylessai", "openai"),
    ("openrouter", "meta-llama/llama-3.3-70b-instruct:free"),
];

const SIMPLE_ROUTE: &[(&str, &str)] = &[
    ("groq", "llama-3.3-70b-versatile"),
    ("cerebras", "gpt-oss-120b"),
    ("sambanova", "Meta-Llama-3.3-70B-Instruct"),
    ("google", "gemini-2.5-flash-lite"),
];

const MEDIUM_ROUTE: &[(&str, &str)] = &[
    ("cerebras", "gpt-oss-120b"),
    ("sambanova", "Meta-Llama-3.3-70B-Instruct"),
    ("google", "gemini-2.5-flash"),
    ("groq", "llama-3.3-70b-versatile"),
];

const OWN_ROUTE: &[(&str, &str)] = &[
    ("anthropic", "claude-opus-5-5"),
    ("copilot", "claude-opus-5.5"),
    ("deepseek", "deepseek-v4-pro"),
    ("openai", "gpt-5.2"),
    ("google", "gemini-2.5-pro"),
];

const COMPLEX_FALLBACK: &[(&str, &str)] = &[
    ("google", "gemini-2.5-flash"),
    ("cerebras", "gpt-oss-120b"),
    ("sambanova", "Meta-Llama-3.3-70B-Instruct"),
    ("groq", "llama-3.3-70b-versatile"),
];

pub const LARGEST: (&str, &str) = ("google", "gemini-2.5-flash");

pub fn chain(task: Task) -> Vec<(&'static str, &'static str)> {
    let head: &[(&str, &str)] = match task {
        Task::Simple => SIMPLE_ROUTE,
        Task::Medium => MEDIUM_ROUTE,
        Task::Complex => COMPLEX_FALLBACK,
    };
    head.iter().chain(FREE_TAIL).copied().collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Ready,
    Wait(u64),
    Spent,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub level: u64,
    pub at: u64,
    pub strikes: u32,
    pub rest: u64,
    pub rpm: u64,
    pub rpd: u64,
    pub recent: Vec<(u64, u64)>,
    pub minutes: Vec<(u64, u64, u64)>,
}

pub const DAY_MINUTES: u64 = 1_440;

pub const SPENT_BACKOFF: u64 = 60;

pub const SPENT_CEILING: u64 = 960;

pub const FAULT_BACKOFF: u64 = 10;

pub const FAULT_CEILING: u64 = 300;

const UNIT: u64 = 1_000;

pub fn backoff(base: u64, strikes: u32, ceiling: u64) -> u64 {
    let doubled = strikes.saturating_sub(1).min(16);
    base.saturating_mul(1u64 << doubled).min(ceiling)
}

impl Account {
    fn fresh(id: &str, quota: Quota, now: u64) -> Account {
        Account {
            id: id.to_string(),
            level: quota.rpm * UNIT,
            at: now,
            ..Account::default()
        }
    }

    pub fn quota(&self, base: Quota) -> Quota {
        Quota {
            rpm: if base.rpm > 0 {
                base.rpm.max(self.rpm)
            } else {
                0
            },
            rpd: if base.rpd > 0 {
                base.rpd.max(self.rpd)
            } else {
                0
            },
            ..base
        }
    }

    fn level_at(&self, quota: Quota, now: u64) -> u64 {
        let full = quota.rpm * UNIT;
        let gained = now.saturating_sub(self.at).saturating_mul(quota.rpm * UNIT) / 60;
        self.level.saturating_add(gained).min(full)
    }

    fn prune(&mut self, now: u64) {
        self.recent.retain(|(second, _)| second + 60 > now);
        let minute = now / 60;
        self.minutes
            .retain(|(held, _, _)| held + DAY_MINUTES > minute);
    }

    fn today(&self, now: u64) -> (u64, u64) {
        let minute = now / 60;
        self.minutes
            .iter()
            .filter(|(held, _, _)| held + DAY_MINUTES > minute)
            .fold((0, 0), |(asked, spent), (_, more, used)| {
                (asked + more, spent + used)
            })
    }

    pub fn verdict(&self, base: Quota, now: u64, need: u64) -> Verdict {
        let quota = self.quota(base);
        if quota.tpm > 0 && need > quota.tpm {
            return Verdict::Spent;
        }
        let (asked, spent) = self.today(now);
        if quota.rpd > 0 && asked >= quota.rpd {
            return Verdict::Spent;
        }
        if quota.tpd > 0 && spent + need > quota.tpd {
            return Verdict::Spent;
        }
        let mut wait = self.rest.saturating_sub(now);
        if quota.rpm > 0 {
            let level = self.level_at(quota, now);
            if level < UNIT {
                let short = UNIT - level;
                wait = wait.max((short * 60).div_ceil(quota.rpm * UNIT));
            }
        }
        if quota.tpm > 0 {
            let mut recent: Vec<(u64, u64)> = self
                .recent
                .iter()
                .copied()
                .filter(|(second, _)| second + 60 > now)
                .collect();
            recent.sort_unstable();
            let mut held: u64 = recent.iter().map(|(_, used)| used).sum();
            for (second, used) in recent {
                if held + need <= quota.tpm {
                    break;
                }
                held -= used;
                wait = wait.max((second + 60).saturating_sub(now));
            }
        }
        if wait > 0 {
            Verdict::Wait(wait)
        } else {
            Verdict::Ready
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub status: u64,
    pub tokens: u64,
    pub retry_after: u64,
    pub limit_minute: u64,
    pub limit_day: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub accounts: Vec<Account>,
}

impl Ledger {
    pub fn account(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|one| one.id == id)
    }

    pub fn verdict(&self, provider: &Provider, now: u64, need: u64) -> Verdict {
        match self.account(provider.id) {
            Some(held) => held.verdict(provider.quota, now, need),
            None => {
                Account::fresh(provider.id, provider.quota, now).verdict(provider.quota, now, need)
            }
        }
    }

    pub fn record(&mut self, id: &str, base: Quota, now: u64, outcome: Outcome) -> u64 {
        let at = match self.accounts.iter().position(|one| one.id == id) {
            Some(at) => at,
            None => {
                self.accounts.push(Account::fresh(id, base, now));
                self.accounts.len() - 1
            }
        };
        let account = &mut self.accounts[at];
        if outcome.limit_minute > base.rpm && base.rpm > 0 {
            account.rpm = outcome.limit_minute;
        }
        if outcome.limit_day > base.rpd && base.rpd > 0 {
            account.rpd = outcome.limit_day;
        }
        let quota = account.quota(base);
        account.level = account.level_at(quota, now);
        account.at = now;
        account.prune(now);
        match outcome.status {
            200..=299 => {
                account.level = account.level.saturating_sub(UNIT);
                account.strikes = 0;
                account.rest = 0;
                account.recent.push((now, outcome.tokens));
                let minute = now / 60;
                match account
                    .minutes
                    .iter_mut()
                    .find(|(held, _, _)| *held == minute)
                {
                    Some(slot) => {
                        slot.1 += 1;
                        slot.2 += outcome.tokens;
                    }
                    None => account.minutes.push((minute, 1, outcome.tokens)),
                }
            }
            429 => {
                account.level = 0;
                account.strikes = account.strikes.saturating_add(1);
                let wait =
                    outcome
                        .retry_after
                        .max(backoff(SPENT_BACKOFF, account.strikes, SPENT_CEILING));
                account.rest = now + wait;
            }
            500..=599 => {
                account.strikes = account.strikes.saturating_add(1);
                let wait =
                    outcome
                        .retry_after
                        .max(backoff(FAULT_BACKOFF, account.strikes, FAULT_CEILING));
                account.rest = now + wait;
            }
            _ => {}
        }
        account.rest.saturating_sub(now)
    }

    pub fn read(text: &str) -> Ledger {
        let Some(held) = json::read(text) else {
            return Ledger::default();
        };
        let number = |from: &Json, key: &str| {
            from.get(key)
                .and_then(Json::integer)
                .map_or(0, |one| one.max(0) as u64)
        };
        let row = |from: &Json| -> Vec<u64> {
            from.list()
                .unwrap_or(&[])
                .iter()
                .map(|one| one.integer().map_or(0, |value| value.max(0) as u64))
                .collect()
        };
        let accounts = held
            .get("accounts")
            .and_then(Json::list)
            .unwrap_or(&[])
            .iter()
            .filter_map(|one| {
                let id = one.get("id").and_then(Json::text)?.to_string();
                let recent = one
                    .get("recent")
                    .and_then(Json::list)
                    .unwrap_or(&[])
                    .iter()
                    .map(row)
                    .filter(|pair| pair.len() == 2)
                    .map(|pair| (pair[0], pair[1]))
                    .collect();
                let minutes = one
                    .get("minutes")
                    .and_then(Json::list)
                    .unwrap_or(&[])
                    .iter()
                    .map(row)
                    .filter(|three| three.len() == 3)
                    .map(|three| (three[0], three[1], three[2]))
                    .collect();
                Some(Account {
                    id,
                    level: number(one, "level"),
                    at: number(one, "at"),
                    strikes: number(one, "strikes").min(u32::MAX as u64) as u32,
                    rest: number(one, "rest"),
                    rpm: number(one, "rpm"),
                    rpd: number(one, "rpd"),
                    recent,
                    minutes,
                })
            })
            .collect();
        Ledger { accounts }
    }

    pub fn write_json(&self, w: &mut Writer) {
        w.begin_object(None);
        w.begin_array(Some("accounts"));
        for account in &self.accounts {
            w.begin_object(None);
            w.field_str("id", &account.id);
            w.field_u64("level", account.level);
            w.field_u64("at", account.at);
            w.field_u64("strikes", u64::from(account.strikes));
            w.field_u64("rest", account.rest);
            w.field_u64("rpm", account.rpm);
            w.field_u64("rpd", account.rpd);
            w.begin_array(Some("recent"));
            for (second, used) in &account.recent {
                w.begin_array(None);
                w.element_u64(*second);
                w.element_u64(*used);
                w.end_array();
            }
            w.end_array();
            w.begin_array(Some("minutes"));
            for (minute, asked, used) in &account.minutes {
                w.begin_array(None);
                w.element_u64(*minute);
                w.element_u64(*asked);
                w.element_u64(*used);
                w.end_array();
            }
            w.end_array();
            w.end_object();
        }
        w.end_array();
        w.end_object();
    }

    pub fn text(&self) -> String {
        let mut w = Writer::new();
        self.write_json(&mut w);
        w.finish()
    }
}

static LEDGER_HELD: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn load(path: &str) -> Ledger {
    if path.is_empty() {
        return Ledger::default();
    }
    std::fs::read_to_string(path)
        .map(|text| Ledger::read(&text))
        .unwrap_or_default()
}

pub fn store(path: &str, ledger: &Ledger) -> bool {
    if path.is_empty() {
        return false;
    }
    let beside = format!("{path}.part");
    let done = std::fs::write(&beside, ledger.text()).and_then(|()| std::fs::rename(&beside, path));
    if done.is_err() {
        std::fs::remove_file(&beside).ok();
    }
    done.is_ok()
}

#[derive(Clone, Copy, Debug)]
pub struct Choice {
    pub provider: &'static Provider,
    pub model: &'static Model,
    pub level: Option<Effort>,
    pub wait: u64,
    pub window: u64,
}

#[derive(Clone, Debug)]
pub struct Route {
    pub task: Task,
    pub choices: Vec<Choice>,
    pub warning: &'static str,
    pub larger: Option<Choice>,
}

pub fn keyed(provider: &Provider, keys: &[String]) -> bool {
    provider.tier == Tier::Keyless || keys.iter().any(|one| one == provider.id)
}

pub fn choice(
    provider: &'static Provider,
    model: &'static Model,
    wanted: Effort,
    wait: u64,
) -> Choice {
    Choice {
        provider,
        model,
        level: settle(model.efforts, wanted),
        wait,
        window: provider.window(model),
    }
}

pub fn route(ask: &Ask, keys: &[String], wanted: Effort, ledger: &Ledger, now: u64) -> Route {
    let task = classify(ask);
    let mut warning = "";
    let mut wanted_chain: Vec<(&str, &str)> = Vec::new();
    if task == Task::Complex {
        let own: Vec<(&str, &str)> = OWN_ROUTE
            .iter()
            .copied()
            .filter(|(id, _)| provider(id).is_some_and(|one| keyed(one, keys)))
            .collect();
        if own.is_empty() {
            warning = "own_key_task";
        }
        wanted_chain.extend(own);
    }
    wanted_chain.extend(chain(task));
    let mut seen: Vec<(&str, &str)> = Vec::new();
    let mut ready = Vec::new();
    let mut waiting = Vec::new();
    let mut narrow = false;
    for (id, model_id) in wanted_chain {
        if seen.contains(&(id, model_id)) {
            continue;
        }
        seen.push((id, model_id));
        let Some(held) = provider(id) else {
            continue;
        };
        let Some(model) = held.model(model_id) else {
            continue;
        };
        if !keyed(held, keys) {
            continue;
        }
        if held.window(model) < ask.tokens {
            narrow = true;
            continue;
        }
        match ledger.verdict(held, now, ask.tokens) {
            Verdict::Ready => ready.push(choice(held, model, wanted, 0)),
            Verdict::Wait(wait) => waiting.push(choice(held, model, wanted, wait)),
            Verdict::Spent => {}
        }
    }
    waiting.sort_by_key(|one: &Choice| one.wait);
    let mut choices = ready;
    choices.extend(waiting);
    if choices.is_empty() && warning.is_empty() {
        warning = "own_key_spent";
    }
    let larger = if narrow {
        larger_than(ask.tokens, &choices, wanted)
    } else {
        None
    };
    Route {
        task,
        choices,
        warning,
        larger,
    }
}

pub fn larger_than(need: u64, choices: &[Choice], wanted: Effort) -> Option<Choice> {
    let held = provider(LARGEST.0)?;
    let model = held.model(LARGEST.1)?;
    let offered = choices
        .iter()
        .any(|one| one.provider.id == held.id && one.window >= need);
    if offered || held.window(model) < need {
        return None;
    }
    Some(choice(held, model, wanted, 0))
}

pub fn fit(provider_id: &str, model_id: &str, window: u64, need: u64, wanted: Effort) -> Json {
    let known = provider(provider_id).and_then(|held| {
        let model = held.model(model_id)?;
        Some(held.window(model))
    });
    let window = known.unwrap_or(window);
    let fits = window == 0 || need <= window;
    let larger = if fits {
        None
    } else {
        larger_than(need, &[], wanted)
    };
    let mut held = vec![
        ("fits".to_string(), Json::Bool(fits)),
        ("window".to_string(), Json::Number(window as f64)),
    ];
    if let Some(larger) = larger {
        held.push((
            "larger".to_string(),
            Json::Object(vec![
                (
                    "provider".to_string(),
                    Json::Text(larger.provider.id.to_string()),
                ),
                ("model".to_string(), Json::Text(larger.model.id.to_string())),
                ("window".to_string(), Json::Number(larger.window as f64)),
            ]),
        ));
    }
    Json::Object(held)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Custom {
    pub name: String,
    pub base: String,
    pub format: Option<Format>,
    pub model: String,
    pub window: u64,
    pub efforts: Vec<Effort>,
    pub template: String,
    pub reply: String,
    pub accepted: bool,
}

pub const NAME_LETTERS: usize = 40;

pub const MODEL_LETTERS: usize = 200;

pub const LEAST_WINDOW: u64 = 1_024;

pub const MOST_WINDOW: u64 = 10_000_000;

pub const CUSTOM_EFFORTS: [Effort; 3] = [Effort::Low, Effort::Medium, Effort::High];

impl Custom {
    pub fn read(held: &Json) -> Custom {
        let text = |key: &str| {
            held.get(key)
                .and_then(Json::text)
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let mut efforts: Vec<Effort> = held
            .get("efforts")
            .map(Json::strings)
            .unwrap_or_default()
            .iter()
            .filter_map(|one| Effort::parse(one))
            .filter(|one| CUSTOM_EFFORTS.contains(one))
            .collect();
        efforts.sort_unstable();
        efforts.dedup();
        Custom {
            name: text("name"),
            base: text("base").trim_end_matches('/').to_string(),
            format: Format::parse(&text("format")).filter(|one| Format::CHOSEN.contains(one)),
            model: text("model"),
            window: held
                .get("window")
                .and_then(Json::integer)
                .map_or(0, |one| one.max(0) as u64),
            efforts,
            template: held
                .get("template")
                .and_then(Json::text)
                .unwrap_or("")
                .to_string(),
            reply: text("reply"),
            accepted: held
                .get("accepted")
                .and_then(Json::boolean)
                .unwrap_or(false),
        }
    }

    pub fn problems(&self) -> Vec<&'static str> {
        let mut found = Vec::new();
        let name_letters = self.name.chars().count();
        if name_letters == 0
            || name_letters > NAME_LETTERS
            || self.name.chars().any(char::is_control)
        {
            found.push("name");
        }
        let secure = host_of(&self.base).is_some_and(|(scheme, host)| {
            scheme == "https" || (scheme == "http" && LOCAL_HOSTS.contains(&host.as_str()))
        });
        if !secure || self.base.chars().any(char::is_whitespace) {
            found.push("base");
        }
        if barred(&self.base).is_some() {
            found.push("barred");
        }
        if self.format.is_none() {
            found.push("format");
        }
        let model_letters = self.model.chars().count();
        if model_letters == 0
            || model_letters > MODEL_LETTERS
            || self.model.chars().any(char::is_whitespace)
        {
            found.push("model");
        }
        if !(LEAST_WINDOW..=MOST_WINDOW).contains(&self.window) {
            found.push("window");
        }
        if self.format == Some(Format::Raw) && !self.template_works() {
            found.push("template");
        }
        if !self.accepted {
            found.push("accepted");
        }
        found
    }

    fn template_works(&self) -> bool {
        let asks = self.template.contains("{{prompt}}") || self.template.contains("{{messages}}");
        if !asks || self.reply.is_empty() {
            return false;
        }
        let filled = fill(
            &self.template,
            &Filling {
                model: "model",
                prompt: "prompt",
                system: "system",
                messages: "[]",
                most_tokens: 1,
                effort: "low",
            },
        );
        json::read(&filled).is_some()
    }

    pub fn wire(&self) -> Wire {
        self.format.map_or(Wire::Prompted, Format::wire)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Filling<'a> {
    pub model: &'a str,
    pub prompt: &'a str,
    pub system: &'a str,
    pub messages: &'a str,
    pub most_tokens: u64,
    pub effort: &'a str,
}

pub fn quoted(text: &str) -> String {
    let mut w = Writer::new();
    w.element_str(text);
    w.finish()
}

pub fn fill(template: &str, filling: &Filling) -> String {
    let messages = if json::read(filling.messages).is_some_and(|held| held.list().is_some()) {
        filling.messages.to_string()
    } else {
        "[]".to_string()
    };
    let pairs = [
        ("{{model}}", quoted(filling.model)),
        ("{{prompt}}", quoted(filling.prompt)),
        ("{{system}}", quoted(filling.system)),
        ("{{messages}}", messages),
        ("{{max_tokens}}", filling.most_tokens.to_string()),
        ("{{effort}}", quoted(filling.effort)),
    ];
    let mut out = String::with_capacity(template.len() + filling.prompt.len());
    let mut rest = template;
    'outer: while !rest.is_empty() {
        if rest.starts_with("{{") {
            for (named, value) in &pairs {
                if let Some(after) = rest.strip_prefix(named) {
                    out.push_str(value);
                    rest = after;
                    continue 'outer;
                }
            }
        }
        let mut letters = rest.chars();
        if let Some(one) = letters.next() {
            out.push(one);
        }
        rest = letters.as_str();
    }
    out
}

pub const DVM_REPLY: &str = "result|response|content|text|output|choices.0.message.content";

pub fn pluck(reply: &Json, path: &str) -> Option<String> {
    path.split('|').find_map(|one| {
        let mut at = reply;
        for step in one.trim().split('.').filter(|step| !step.is_empty()) {
            at = match (step.parse::<usize>(), at) {
                (Ok(index), Json::List(held)) => held.get(index)?,
                _ => at.get(step)?,
            };
        }
        match at {
            Json::Text(held) => Some(held.clone()),
            Json::Number(held) => Some(held.to_string()),
            Json::Bool(held) => Some(held.to_string()),
            _ => None,
        }
    })
}

fn write_setting(w: &mut Writer, key: Option<&str>, held: &Setting) {
    w.begin_object(key);
    w.field_str("level", held.level.name());
    w.field_str("param", held.param);
    w.field_str("value", &held.value);
    w.field_u64("mostTokens", held.most_tokens);
    w.field_str("hint", held.hint);
    w.end_object();
}

fn write_efforts(w: &mut Writer, efforts: &[Effort], wire: Wire) {
    w.begin_array(Some("efforts"));
    for level in efforts {
        write_setting(w, None, &setting(wire, *level));
    }
    w.end_array();
}

fn write_quota(w: &mut Writer, quota: Quota) {
    w.begin_object(Some("quota"));
    w.field_u64("rpm", quota.rpm);
    w.field_u64("rpd", quota.rpd);
    w.field_u64("tpm", quota.tpm);
    w.field_u64("tpd", quota.tpd);
    w.end_object();
}

pub fn probe_of(held: &Provider) -> (&'static str, &'static str) {
    match held.id {
        "openrouter" => ("GET", "/key"),
        _ => held.format.probe(),
    }
}

fn write_probe(w: &mut Writer, (method, path): (&str, &str)) {
    w.begin_object(Some("probe"));
    w.field_str("method", method);
    w.field_str("path", path);
    w.end_object();
}

pub fn write_provider(w: &mut Writer, held: &Provider) {
    w.begin_object(None);
    w.field_str("id", held.id);
    w.field_str("shown", held.shown);
    w.field_str("tier", held.tier.name());
    w.field_str("format", held.format.name());
    w.field_str("sign", held.sign.name());
    w.field_str("base", held.base);
    w.field_str("home", held.home);
    w.field_str("privacy", held.privacy.name());
    w.field_str("notice", held.notice);
    w.field_bool("lastResort", held.last_resort);
    w.field_u64("freeWindow", held.free_window);
    write_quota(w, held.quota);
    write_probe(w, probe_of(held));
    w.begin_array(Some("models"));
    for model in held.models {
        w.begin_object(None);
        w.field_str("id", model.id);
        w.field_str("shown", model.shown);
        w.field_u64("window", held.window(model));
        w.field_u64("fullWindow", model.window);
        write_efforts(w, model.efforts, model.wire);
        w.end_object();
    }
    w.end_array();
    w.end_object();
}

pub fn catalog_json() -> String {
    let mut w = Writer::new();
    w.begin_object(None);
    w.begin_array(Some("providers"));
    for held in PROVIDERS {
        write_provider(&mut w, held);
    }
    w.end_array();
    w.begin_array(Some("barred"));
    for held in BARRED {
        w.begin_object(None);
        w.field_str("name", held.name);
        w.field_str("why", held.why);
        w.begin_array(Some("hosts"));
        for host in held.hosts {
            w.element_str(host);
        }
        w.end_array();
        w.end_object();
    }
    w.end_array();
    w.begin_array(Some("formats"));
    for format in Format::CHOSEN {
        w.element_str(format.name());
    }
    w.end_array();
    w.end_object();
    w.finish()
}

pub const LISTED_WINDOW: u64 = 16_000;

pub const UNSPOKEN: &[&str] = &[
    "embed",
    "whisper",
    "tts",
    "dall-e",
    "moderation",
    "image",
    "audio",
    "transcribe",
    "realtime",
    "rerank",
    "guard",
    "imagen",
    "veo",
    "aqa",
    "safety",
    "lyria",
    "sora",
    "speech",
    "music",
    "video",
    "ocr",
    "search-preview",
    "deep-research",
    "computer-use",
    "davinci",
    "babbage",
    "diffusion",
    "flux",
    "kling",
    "seedream",
    "seedance",
    "upscale",
];

pub const PAID_TIERS: &[&str] = &["seed", "flower", "nectar", "paid"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Listed {
    pub id: String,
    pub window: u64,
}

fn amount(held: Option<&Json>) -> Option<f64> {
    match held? {
        Json::Number(said) => Some(*said),
        Json::Text(said) => said.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn free_priced(one: &Json) -> Option<bool> {
    if let Some(said) = one.get("isFree").and_then(Json::boolean) {
        return Some(said);
    }
    if let Some(billed) = one.get("usage_based_only").and_then(Json::boolean) {
        return Some(!billed);
    }
    let pricing = one.get("pricing")?;
    let costs: Vec<f64> = ["prompt", "completion", "input", "output"]
        .iter()
        .filter_map(|named| amount(pricing.get(named)))
        .collect();
    if costs.is_empty() {
        return None;
    }
    Some(costs.iter().all(|cost| *cost == 0.0))
}

fn spoken_only(one: &Json) -> bool {
    let outputs = one
        .get("architecture")
        .and_then(|held| held.get("output_modalities"))
        .or_else(|| one.get("modalities").and_then(|held| held.get("output")))
        .or_else(|| one.get("output_modalities"));
    if let Some(outputs) = outputs.and_then(Json::list) {
        let said: Vec<&str> = outputs.iter().filter_map(Json::text).collect();
        if !said.contains(&"text") || said.iter().any(|kind| *kind != "text") {
            return false;
        }
    }
    if let Some(kind) = one.get("model_type").and_then(Json::text) {
        if kind != "chat" {
            return false;
        }
    }
    if let Some(kind) = one
        .get("capabilities")
        .and_then(|held| held.get("type"))
        .and_then(Json::text)
    {
        if kind != "chat" {
            return false;
        }
    }
    if let Some(methods) = one.get("supportedGenerationMethods").and_then(Json::list) {
        if !methods
            .iter()
            .any(|method| method.text() == Some("generateContent"))
        {
            return false;
        }
    }
    true
}

fn window_of(one: &Json) -> u64 {
    let found = one
        .get("context_length")
        .and_then(Json::integer)
        .or_else(|| {
            one.get("context_window").and_then(|held| {
                held.integer()
                    .or_else(|| held.get("tokens").and_then(Json::integer))
            })
        })
        .or_else(|| one.get("inputTokenLimit").and_then(Json::integer))
        .or_else(|| one.get("max_context_length").and_then(Json::integer))
        .or_else(|| {
            one.get("top_provider")
                .and_then(|held| held.get("context_length"))
                .and_then(Json::integer)
        })
        .or_else(|| {
            one.get("capabilities")
                .and_then(|held| held.get("limits"))
                .and_then(|held| held.get("max_context_window_tokens"))
                .and_then(Json::integer)
        });
    found.map_or(0, |said| said.max(0) as u64)
}

pub fn usable_models(tier: Tier, reply: &Json) -> Vec<Listed> {
    let items: &[Json] = match reply {
        Json::List(items) => items,
        _ => reply
            .get("data")
            .or_else(|| reply.get("models"))
            .and_then(Json::list)
            .unwrap_or(&[]),
    };
    let mut out: Vec<Listed> = Vec::new();
    for one in items {
        let named = one
            .get("id")
            .and_then(Json::text)
            .filter(|said| !said.is_empty())
            .or_else(|| one.get("name").and_then(Json::text))
            .unwrap_or("");
        let id = named.strip_prefix("models/").unwrap_or(named).trim();
        if id.is_empty() {
            continue;
        }
        let lower = id.to_ascii_lowercase();
        if UNSPOKEN.iter().any(|word| lower.contains(word)) || !spoken_only(one) {
            continue;
        }
        let window = window_of(one);
        if window > 0 && window < LISTED_WINDOW {
            continue;
        }
        if tier != Tier::Own {
            if free_priced(one) == Some(false) {
                continue;
            }
            if let Some(level) = one.get("tier").and_then(Json::text) {
                if PAID_TIERS.contains(&level) {
                    continue;
                }
            }
            if one
                .get("billing")
                .and_then(|held| held.get("is_premium"))
                .and_then(Json::boolean)
                == Some(true)
            {
                continue;
            }
        }
        if out.iter().all(|held| held.id != id) {
            out.push(Listed {
                id: id.to_string(),
                window,
            });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn listed_json(held: &[Listed]) -> String {
    let mut w = Writer::new();
    w.begin_object(None);
    w.begin_array(Some("models"));
    for one in held {
        w.begin_object(None);
        w.field_str("id", &one.id);
        w.field_u64("window", one.window);
        w.end_object();
    }
    w.end_array();
    w.end_object();
    w.finish()
}

fn write_choice(w: &mut Writer, key: Option<&str>, held: &Choice) {
    w.begin_object(key);
    w.field_str("provider", held.provider.id);
    w.field_str("model", held.model.id);
    w.field_u64("window", held.window);
    w.field_u64("wait", held.wait);
    match held.level {
        Some(level) => {
            w.field_str("effort", level.name());
            write_setting(w, Some("setting"), &setting(held.model.wire, level));
        }
        None => {
            w.field_str("effort", "");
        }
    }
    w.end_object();
}

pub fn route_json(held: &Route) -> String {
    let mut w = Writer::new();
    w.begin_object(None);
    w.field_str("task", held.task.name());
    w.field_str("warning", held.warning);
    w.begin_array(Some("choices"));
    for one in &held.choices {
        write_choice(&mut w, None, one);
    }
    w.end_array();
    if let Some(larger) = &held.larger {
        write_choice(&mut w, Some("larger"), larger);
    }
    w.end_object();
    w.finish()
}

pub fn custom_json(held: &Custom) -> String {
    let problems = held.problems();
    let format = held.format.unwrap_or(Format::Open);
    let mut w = Writer::new();
    w.begin_object(None);
    w.field_bool("ok", problems.is_empty());
    w.begin_array(Some("problems"));
    for one in &problems {
        w.element_str(one);
    }
    w.end_array();
    w.field_str("format", format.name());
    w.field_str("sign", format.sign().name());
    w.field_str("base", &held.base);
    w.field_u64("window", held.window);
    write_probe(&mut w, format.probe());
    write_efforts(&mut w, &held.efforts, held.wire());
    if let Some(found) = barred(&held.base) {
        w.field_str("barred", found.name);
    }
    w.end_object();
    w.finish()
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn number_of(held: &Json, key: &str) -> u64 {
    held.get(key)
        .and_then(Json::integer)
        .map_or(0, |one| one.max(0) as u64)
}

fn text_of<'a>(held: &'a Json, key: &str) -> &'a str {
    held.get(key).and_then(Json::text).unwrap_or("")
}

fn wanted_of(held: &Json) -> Effort {
    Effort::parse(text_of(held, "effort")).unwrap_or(Effort::High)
}

fn when(held: &Json) -> u64 {
    match number_of(held, "now") {
        0 => now_seconds(),
        said => said,
    }
}

fn refused(why: &str) -> String {
    let mut w = Writer::new();
    w.begin_object(None);
    w.field_str("error", why);
    w.end_object();
    w.finish()
}

pub fn answer(request: &str) -> String {
    let Some(held) = json::read(request) else {
        return refused("request");
    };
    match text_of(&held, "op") {
        "catalog" => catalog_json(),
        "route" => {
            let ask = Ask {
                tokens: number_of(&held, "tokens"),
                files: number_of(&held, "files"),
                code: held.get("code").and_then(Json::boolean).unwrap_or(false),
                failure: held.get("failure").and_then(Json::boolean).unwrap_or(false),
            };
            let keys = held.get("keyed").map(Json::strings).unwrap_or_default();
            let ledger = {
                let _held = LEDGER_HELD
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                load(text_of(&held, "ledger"))
            };
            route_json(&route(&ask, &keys, wanted_of(&held), &ledger, when(&held)))
        }
        "record" => {
            let id = text_of(&held, "provider");
            let base = provider(id).map_or(OPEN, |known| known.quota);
            let outcome = Outcome {
                status: number_of(&held, "status"),
                tokens: number_of(&held, "tokens"),
                retry_after: number_of(&held, "retryAfter"),
                limit_minute: number_of(&held, "limitMinute"),
                limit_day: number_of(&held, "limitDay"),
            };
            let path = text_of(&held, "ledger");
            let _held = LEDGER_HELD
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut ledger = load(path);
            let rest = ledger.record(id, base, when(&held), outcome);
            let kept = store(path, &ledger);
            let mut w = Writer::new();
            w.begin_object(None);
            w.field_u64("rest", rest);
            w.field_bool("kept", kept);
            w.end_object();
            w.finish()
        }
        "custom" => custom_json(&Custom::read(&held)),
        "models" => {
            let tier = match text_of(&held, "tier") {
                "byok" | "own" => Tier::Own,
                "keyless" => Tier::Keyless,
                "custom" => Tier::Own,
                _ => Tier::Free,
            };
            let reply = json::read(text_of(&held, "reply")).unwrap_or(Json::Null);
            listed_json(&usable_models(tier, &reply))
        }
        "fit" => {
            let verdict = fit(
                text_of(&held, "provider"),
                text_of(&held, "model"),
                number_of(&held, "window"),
                number_of(&held, "tokens"),
                wanted_of(&held),
            );
            let mut w = Writer::new();
            write_value(&mut w, None, &verdict);
            w.finish()
        }
        "fill" => {
            let body = fill(
                text_of(&held, "template"),
                &Filling {
                    model: text_of(&held, "model"),
                    prompt: text_of(&held, "prompt"),
                    system: text_of(&held, "system"),
                    messages: text_of(&held, "messages"),
                    most_tokens: number_of(&held, "maxTokens"),
                    effort: text_of(&held, "level"),
                },
            );
            let mut w = Writer::new();
            w.begin_object(None);
            w.field_bool("ok", json::read(&body).is_some());
            w.field_str("body", &body);
            w.end_object();
            w.finish()
        }
        "pluck" => {
            let path = match text_of(&held, "path") {
                "" => DVM_REPLY,
                said => said,
            };
            let found = json::read(text_of(&held, "reply")).and_then(|reply| pluck(&reply, path));
            let mut w = Writer::new();
            w.begin_object(None);
            w.field_bool("found", found.is_some());
            w.field_str("text", found.as_deref().unwrap_or(""));
            w.end_object();
            w.finish()
        }
        _ => refused("op"),
    }
}

fn write_value(w: &mut Writer, key: Option<&str>, held: &Json) {
    match held {
        Json::Object(pairs) => {
            w.begin_object(key);
            for (named, value) in pairs {
                write_value(w, Some(named), value);
            }
            w.end_object();
        }
        Json::List(items) => {
            w.begin_array(key);
            for value in items {
                write_value(w, None, value);
            }
            w.end_array();
        }
        Json::Text(said) => match key {
            Some(named) => w.field_str(named, said),
            None => w.element_str(said),
        },
        Json::Bool(said) => match key {
            Some(named) => w.field_bool(named, *said),
            None => w.element_str(if *said { "true" } else { "false" }),
        },
        Json::Number(said) => match key {
            Some(named) => w.field_u64(named, said.max(0.0) as u64),
            None => w.element_u64(said.max(0.0) as u64),
        },
        Json::Null => match key {
            Some(named) => w.field_str(named, ""),
            None => w.element_str(""),
        },
    }
}
