pub const DOCUMENTS: &[(&str, &str)] = &[
    ("en", ENGLISH),
    ("tr", TURKISH),
    ("de", GERMAN),
    ("es", SPANISH),
    ("fr", FRENCH),
    ("it", ITALIAN),
    ("pt", PORTUGUESE),
    ("ru", RUSSIAN),
    ("ar", ARABIC),
    ("zh", CHINESE),
    ("ja", JAPANESE),
    ("ko", KOREAN),
    ("hi", HINDI),
    ("id", INDONESIAN),
    ("pt-BR", BRAZILIAN),
];

const ENGLISH: &str = r#"@@ Logcat Commands
1- Adb Logcat

Streams every log on the device live into the terminal. From the moment the command runs, every log line produced by the system, apps, services and the kernel lands on the screen. Press Ctrl + C on the keyboard to stop it. This command is the foundation of debugging; but used without a filter it can pour out hundreds of lines a second, which makes finding what you need hard.

Example usage:

```
adb logcat
```

Example output:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Dumps the logs already collected onto the screen and the command ends. Instead of a continuous stream it takes a snapshot of that moment. Use it when writing a bug report, when saving the state after a crash, or when you want a one-off log from inside a script.

Example usage:

```
adb logcat -d
```

Example usage (saving to a file):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

This command is used a lot in scripts that send reports to a server automatically, because it ends on its own and never blocks the script.

---

3- Adb Logcat -C

Clears every log buffer, deleting everything collected so far. Run it before starting a new debugging session and you only see logs produced from that moment on, so old noise cannot mislead you. When you are trying to reproduce a bug, this is usually the first step.

Example usage:

```
adb logcat -c
adb logcat
```

Running the two commands above in order gives a perfectly clean log stream. Only what happens after the clear reaches the screen.

---

4- Adb Logcat -G

Shows the size of the current log buffers and how full they are. It lists, line by line, how full each buffer is, its total capacity and the amount used. If logs are flowing very fast and old entries are being dropped (this is called "chatty"), look at this first.

Example usage:

```
adb logcat -g
```

Example output:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Size

Changes the size of the log buffer. For example you can turn a 256 KiB buffer into 16 MiB. More logs are kept that way and old entries are dropped later. Very useful during long tests or while debugging a very noisy app. The setting resets when the device restarts.

Example usage:

```
adb logcat -G 16M
```

Example usage (to shrink it):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F File

Writes the logs to the given file instead of printing them to the terminal. The terminal stays clean and the record is kept. The file path is on the device, not on a computer. When working in a mobile terminal (Termux) it is usually written under /Sdcard/.

Example usage:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Example usage (appending to the file):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

In the example above a new file is opened when the file reaches 4 MiB, and 10 files are kept in total.

---

7- Adb Logcat -B Main

Watches only the main app buffer. The logs apps write with calls such as Log.d, Log.i and Log.e live here. It is watched by default anyway, but you name it explicitly when you want only this buffer. It is where you look most while developing an app.

Example usage:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Watches the logs of system services. The logs written by services such as ActivityManager, WindowManager, PackageManager and PowerManager live here. Use it to see events such as app starts, permission grants and service bindings. It is usually watched together with main.

Example usage:

```
adb logcat -b system
```

Example output:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Watches only crash logs. The FATAL EXCEPTION blocks written when an app force closes live here. When debugging a crash, looking at this buffer first is the fastest route because the stack trace is right there.

Example usage:

```
adb logcat -b crash
```

Example output:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Watches logs about radio, telephony, signal and data connectivity. Use it when dealing with the SIM card, network registration, 4G/5G connectivity, SMS or calls. It is usually not needed for ordinary app development.

Example usage:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Watches the binary logs of system events. This buffer is not plain text; it holds short encoded records left by Log.event calls. They appear with tags such as am_proc_start and am_activity_launch. Used in performance analysis.

Example usage:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Watches kernel logs. Shows the messages the Linux kernel writes; this usually resembles the output of the dmesg command. Driver errors, memory problems, heat and power management information live here. It may require root.

Example usage:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Watches every buffer at once. Main, system, crash, radio, events and kernel all flow mixed together. Use it when you want to see everything, but the output gets very crowded, so it is usually combined with a filter.

Example usage:

```
adb logcat -b all
```

Example usage (filtered):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Several Buffers

Watches several comma-separated buffers at the same time. For example, to see app and crash logs together. By choosing only the channels you need you can narrow the output.

Example usage:

```
adb logcat -b main,crash
```

Example usage (three channels):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

The default short output format. Each line starts only with the priority level and the tag. Good for a quick look, but with no timestamp it is hard to order events.

Example usage:

```
adb logcat -v brief
```

Example output:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Adds a date and time stamp. Far more useful than the short format because you see when an event happened. Milliseconds are shown below the second.

Example usage:

```
adb logcat -v time
```

Example output:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

The most recommended output format. It carries the date, time, priority, tag, PID (process id) and TID (thread id). In multithreaded apps it is essential for telling which thread did what. It is also the format Android Studio uses by default.

Example usage:

```
adb logcat -v threadtime
```

Example output:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Gives detailed output with every metadata field and a blank line between entries. Used in long sessions to see each log as a block, but it takes up space.

Example usage:

```
adb logcat -v long
```

Example output:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colours logs by priority level. Errors turn red, warnings orange, info green and debug blue. In a long stream it helps your eye go straight to the error. The terminal needs colour support.

Example usage:

```
adb logcat -v color
```

Example usage (together with time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Adds the year to the time format. Used in long-running records or archives that span more than one year. Usually not needed in ordinary development.

Example usage:

```
adb logcat -v year
```

Example output:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Shows the user id (UID) of the app that produced the log. Used to tell which app wrote a line when several apps use the same tag. It is a modifier, so it is combined with the other formats.

Example usage:

```
adb logcat -v uid
```

Example output:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Tag And Priority Filter

Shows logs with a given tag at or above a given priority level. The priority levels are: V (Verbose) the lowest, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) and S (Silent) the highest. This way of filtering is logcat's most powerful feature.

Example usage (errors only):

```
adb logcat *:E
```

Example usage (warnings and above):

```
adb logcat *:W
```

Example usage (allow list):

```
adb logcat ActivityManager:I MyApp:D *:S
```

The last example above shows ActivityManager at info and above and MyApp at debug and above, and silences every other tag. In some shells the * character is special, so it must be quoted:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Tag

A shortcut for the silent filter. It shows only the given tag and silences everything else. The most practical way to focus on your own tag while developing an app.

Example usage:

```
adb logcat -s MyApp
```

Example usage (several tags):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

In the example above only the logs of three tags flow; everything else stays silent.

---

24- Adb Logcat -S With A Level

Used to give both the tag and the priority level together. For example, to see only warnings and above from the MyApp tag.

Example usage:

```
adb logcat MyApp:W *:S
```

This command shows only warning, error and fatal logs tagged MyApp.

---

25- Adb Logcat --Pid Filter

Shows only the logs belonging to a given process id (PID). To learn an app's PID, adb shell pidof is used first, then the logs are filtered by that PID. Perfect for isolating every log of one session.

Example usage (getting the PID):

```
adb shell pidof com.example.app
```

Example output:

```
5678
```

Example usage (filtering):

```
adb logcat --pid=5678
```

Example usage (on one line):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Time

Shows logs from the given timestamp onwards. In other words it says "bring me what comes after this moment". It is written with a capital T; a lowercase t means something else.

Example usage:

```
adb logcat -T "09-21 14:00:00.000"
```

Example usage (from today onwards):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Line Count

With a lowercase t it shows the given number of most recent lines and the command ends. It behaves like -d but gives only the last N lines. Ideal for a quick look.

Example usage (last 100 lines):

```
adb logcat -t 100
```

Example usage (last 500 lines and a filter):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Searches the content with a regular expression (regex). Shows the lines whose content matches the given pattern. Very powerful for fast text search; especially when you look for several words at once it is quicker than filtering.

Example usage (one word):

```
adb logcat -e "Login"
```

Example usage (several words):

```
adb logcat -e "Login|Error|Crash"
```

Example usage (with a package name):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Binary Output

Writes the logs in binary format. People cannot read this format, but another logcat can read it back or other tools can process it. Used in automatic analysis systems.

Example usage:

```
adb logcat -B > Ham_Log.Bin
```

That file is read back like this later:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Statistics

Shows log statistics: how many logs each tag produced and which tags triggered the "chatty" warning. If you think an app is spamming the log, you can confirm it with this command.

Example usage:

```
adb logcat -S
```

Example output:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R And -N Together

Used for rotating logs. -r sets the size of each file and -n how many files are kept. Use it when an app will run for a long time and you want a log kept continuously. Old files are deleted and new ones opened.

Example usage:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

In the example above a new file is opened each time a file reaches 4 MiB and 10 files are kept in total, so the disk never fills up without end.

---

32- Adb Logcat -P Filter Script

Names a custom filter script for logcat. This is Android's own filter format and is usually used by system developers. Ordinary app developers rarely use it.

Example usage:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Timestamps

An option used in some formats to add a timestamp to the start of each log line. Its behaviour changes between Android versions. It usually does the same job as -v time, but in a different shape.

Example usage:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Shows each line in printf format, that is priority (PID) tag: message. Used for compatibility with some older tools, but rarely needed on modern Android.

Example usage:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Shows only the raw message and adds no metadata. Only the log itself is written, with no tag, priority or timestamp. Used when passing output to another program.

Example usage:

```
adb logcat -v raw
```

Example output:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Writes only the tag at the start of each line, then the rest. The priority level and PID are hidden. Handy when you are focused on one tag.

Example usage:

```
adb logcat -v tag
```

Example output:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Writes the PID first and then the tag at the start of each line. Handy for per-process analysis. Good for seeing quickly which process wrote what.

Example usage:

```
adb logcat -v process
```

Example output:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Writes only the thread id (TID) at the start of each line, then the rest. Used in multithreaded analysis to see which thread is active. A short format.

Example usage:

```
adb logcat -v thread
```

Example output:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D And -T Together

Used to take the logs of a time range. For example, to take the logs of the last hour and save them to a file.

Example usage:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Example usage (to a file):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C And -D In Order

A flow that clears and then collects. For example, clear before starting a test and take the logs when the test ends.

Example usage:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Saved To A File And Pulled To A Computer

Handy for keeping a long record in a mobile terminal. It is first written to a file on the device and then pulled to the computer with adb pull.

Example usage (save on the device):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Example usage (pull to the computer):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat With Grep

Used in a mobile terminal (Termux) for fast text search with grep. It can be more flexible than the regex filter because you can use all of grep's options.

Example usage:

```
adb logcat | grep "Hata"
```

Example usage (case insensitive):

```
adb logcat | grep -I "error"
```

Example usage (with line numbers):

```
adb logcat | grep -N "Exception"
```

Example usage (several words):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat With Tail

Used to cut the live stream down to its last few lines, when you want to see only the end instead of the whole log.

Example usage:

```
adb logcat | tail -100
```

Example usage (follow continuously):

```
adb logcat | tail -F
```

---

44- Adb Logcat With Head

Used to see the first few lines. Good for a quick look at a long record.

Example usage:

```
adb logcat -d | head -50
```

---

45- Adb Logcat With Wc

Used to see the total number of lines. For example, to measure how many logs were produced during a test.

Example usage:

```
adb logcat -d | wc -l
```

Example usage (count only the errors):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat With Sort

Used to sort logs alphabetically or numerically. Usually used during analysis.

Example usage:

```
adb logcat -d | sort
```

Example usage (reverse order):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat With Uniq

Used to remove repeated lines. If an app keeps writing the same error, this command simplifies the output.

Example usage:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

The example above counts and shows the 20 most repeated lines. Very useful in debugging.

---

48- Adb Logcat With Awk

Used to take given columns or to work on the text, such as taking the PID column in threadtime format.

Example usage (only the PID column):

```
adb logcat -v threadtime | awk '{print $3}'
```

Example usage (only the tag and the message):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat With Xargs

Used to run a command for each line, such as sending a notification for every error line. Usually used in automation.

Example usage:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat With Sed

Used to replace text. Ideal for changing given patterns in the output.

Example usage:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

The example above writes Uygulama in place of every MyApp.

---

51- Adb Logcat With Tee

Used to show on screen and write to a file at the same time. Live watching and recording happen together.

Example usage:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat With Watch

Used to take logs at set intervals, for example a log summary every 5 seconds.

Example usage:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat With Timeout

Used to stop logcat after a set time. Very useful in automated tests.

Example usage:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat With Alias

Used to shorten the commands you use often. In Termux it is added to the .bashrc or .zshrc file.

Example usage (define an alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Then you only call it like this:

```
lce
```

---

55- Adb Logcat With Several Devices

Used to target one device when several are connected. The devices are listed first with adb devices, then one is chosen with -s.

Example usage:

```
adb devices
```

Example output:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Example usage (a given device):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat With Wireless Debugging

Used for wireless debugging. When the device is on the same Wi-Fi network you connect without a cable. Android 11 and later support it.

Example usage (turn on wireless debugging on the device, then):

```
adb pair 192.168.1.42:37123
```

Example usage (connect):

```
adb connect 192.168.1.42:5555
```

Example usage (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat With Bugreport

Used to take a detailed report of the whole system. Next to logcat, many tools such as dumpsys, dmesg and top are run and collected into one zip. The most thorough way to debug.

Example usage:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Example usage (pull to the computer):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat With Dumpsys

Used to see the current state of system services next to logcat, for example what ActivityManager is doing right now.

Example usage:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat With ANR Traces

Used to take the stack traces when an app not responding (ANR) error happens. ANR records live in the /Data/Anr/Traces.Txt file.

Example usage:

```
adb shell cat /Data/Anr/Traces.Txt
```

Example usage (needs root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat With Tombstones

Used for native crashes. A file is created in the /Data/Tombstones/ folder for each crash. Look here when there is a fault in C++ or Rust code.

Example usage:

```
adb shell ls /Data/Tombstones/
```

Example usage (the latest tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat With Systrace

Used for system-wide performance tracing. It collects tracing data at kernel level, not only logs. Used together with Perfetto.

Example usage:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat With Layout Inspector

Used to analyse an app's layout on screen. It is used with Layout Inspector, but you can also follow the related logs with logcat.

Example usage:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat With Memory Profiler

Used to watch memory use. Memory Profiler usually lives in Android Studio, but you can also see memory-related logs in logcat.

Example usage:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat With CPU Profiler

Used to watch CPU use. For a simple measurement the top command can be used too.

Example usage:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat With Network Profiler

Used to watch network traffic. Libraries such as OkHttp and Retrofit write to logcat, but a full analysis needs Network Profiler.

Example usage:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat With GPU Profiler

Used to watch GPU use. For a simple measurement the gfxinfo command can be used too.

Example usage:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat With Battery Historian

Used to watch battery use. You can also look with dumpsys battery.

Example usage:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat For Wi-Fi Diagnosis

Used to debug Wi-Fi problems. Look here for dropped connections, a weak signal and the like.

Example usage:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat For Bluetooth Diagnosis

Used to debug Bluetooth problems. Look here for pairing problems, disconnections and the like.

Example usage:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat For Sensor Diagnosis

Used to debug sensor problems, for sensors such as the accelerometer, gyroscope and magnetometer.

Example usage:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat For Camera Diagnosis

Used to debug camera problems, for cases such as taking photos and recording video.

Example usage:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat For Audio Diagnosis

Used to debug audio problems, for cases such as sound not playing or the microphone not working.

Example usage:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat For Location Diagnosis

Used to debug location problems, for cases such as no GPS signal or a location that does not update.

Example usage:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat For Notification Diagnosis

Used to debug notification problems, for cases such as notifications not appearing or arriving at the wrong time.

Example usage:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat For Permission Diagnosis

Used to debug permission problems, for cases such as a permission not being granted or being refused.

Example usage:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat For WebView Diagnosis

Used to debug WebView problems, for cases such as a page not loading or a JavaScript error.

Example usage:

```
adb logcat -s Chromium WebView
```

Example usage (more detail):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat For OkHttp Diagnosis

Used to debug OkHttp problems: network requests, responses and errors.

Example usage:

```
adb logcat -s Okhttp
```

Example usage (every level):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat For Retrofit Diagnosis

Used to debug Retrofit problems. Since it is usually built on OkHttp, its logs are the ones to look at.

Example usage:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat For Glide Diagnosis

Used to debug Glide (image loading library) problems, for cases such as images not loading or memory trouble.

Example usage:

```
adb logcat -s Glide
```

---

80- Adb Logcat For Picasso Diagnosis

Used to debug Picasso (image loading library) problems.

Example usage:

```
adb logcat -s Picasso
```

---

81- Adb Logcat For Room Diagnosis

Used to debug Room (database library) problems: query errors and schema problems.

Example usage:

```
adb logcat -s Room
```

---

82- Adb Logcat For Firebase Diagnosis

Used to debug Firebase problems, for modules such as Crashlytics, Analytics and Messaging.

Example usage:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat For Crashlytics Diagnosis

Used to debug Firebase Crashlytics problems.

Example usage:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat For Analytics Diagnosis

Used to debug analytics problems: events not being sent and batch sending problems.

Example usage:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat With Wireless ADB Debugging

Wireless debugging sometimes needs adb restarted. Connect over USB first, then switch to wireless.

Example usage:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat While Restarting The Device

Used to restart the device and watch the logs, especially for faults that happen during boot.

Example usage (two terminals):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat While Restarting An App

Used to restart just one app and watch the logs.

Example usage:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat While Installing An App

Used to install an APK and watch the logs.

Example usage:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat While Uninstalling An App

Used to uninstall an app and watch the logs.

Example usage:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat With A Screenshot

Used to take a screenshot and save the log at the moment of a fault.

Example usage:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat With A Screen Recording

Used to record the screen and save the log at the moment of a fault. Android 4.4 and later support it.

Example usage:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat With A Monkey Test

Used to test an app by simulating random user interaction. Very effective together with the logs.

Example usage:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat With A UI Automator Test

Used for UI automation tests, to watch the logs while the test runs.

Example usage:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat With An Espresso Test

Used for Espresso tests, a subset of UI automation.

Example usage:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat With A JUnit Test

Used for unit tests. They usually run on a computer but can run on the device too.

Example usage:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat With StrictMode

Used to catch app performance problems. It warns when disk or network work is done on the main thread.

Example usage:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat With Choreographer

Used to catch skipped frame problems. Look here when animations stutter.

Example usage:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat With GPU Overdraw

Used to catch overdraw problems. Used in visual performance analysis.

Example usage:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat With The Render Thread

Used to catch render thread problems. On modern Android drawing happens on this thread.

Example usage:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat With The GPU Renderer

Used to catch GPU rendering problems. Look here once Profile GPU Rendering is turned on.

Example usage:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat With TextView

Used to catch problems with text views, such as font problems and measuring errors.

Example usage:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat With RecyclerView

Used to catch problems with list views: scrolling performance and view recycling.

Example usage:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat With ViewPager

Used to catch problems with page transitions.

Example usage:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat With Fragments

Used to catch fragment lifecycle problems.

Example usage:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat With Activities

Used to catch activity lifecycle problems.

Example usage:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat With Services

Used to catch problems with background services.

Example usage:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat With WorkManager

Used to catch WorkManager problems, for background tasks.

Example usage:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat With AlarmManager

Used to catch AlarmManager problems, for scheduled tasks.

Example usage:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat With SharedPreferences

Used to catch problems with SharedPreferences (simple data storage).

Example usage:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat With SQLite

Used to catch problems with SQLite (the local database).

Example usage:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat With File Operations

Used to catch problems with reading and writing files.

Example usage:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat With Network Operations

Used to catch network connection problems, such as HTTP, TCP and UDP.

Example usage:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat With SSL

Used to catch SSL certificate problems. Look here when secure connections fail.

Example usage:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat With JSON

Used to catch JSON parsing problems.

Example usage:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat With XML

Used to catch XML parsing problems.

Example usage:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat With Images

Used to catch image loading and processing problems.

Example usage:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat With Media

Used to catch media playback and recording problems.

Example usage:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat With ExoPlayer

Used to catch ExoPlayer (media player library) problems.

Example usage:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat With Chromecast

Used to catch Chromecast problems.

Example usage:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat With Ads

Used to catch problems with advertising libraries.

Example usage:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat With Billing

Used to catch Google Play billing problems.

Example usage:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat With Maps

Used to catch problems with map libraries.

Example usage:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat With Authentication

Used to catch sign-in problems, such as Google Sign-In and Firebase Auth.

Example usage:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat With Notifications

Used to catch problems sending and receiving notifications.

Example usage:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat With Deep Links

Used to catch deep link problems, such as opening an app from a URL.

Example usage:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat With Intents

Used to catch problems with intents, for moving between apps.

Example usage:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat With Broadcasts

Used to catch problems with broadcast messages between the system and apps.

Example usage:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat With Content Providers

Used to catch content provider problems, for sharing data between apps.

Example usage:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat With JobScheduler

Used to catch JobScheduler problems, for scheduled background tasks.

Example usage:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat With Doze Mode

Used to catch Doze mode (sleep mode) problems, for background work while the device sleeps.

Example usage:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat With Battery Optimisation

Used to catch battery optimisation problems, for restrictions on running in the background.

Example usage:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat With Memory Leaks

Used to catch memory leak problems. Used together with tools such as LeakCanary.

Example usage:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat With ANRs

Used to catch ANR (application not responding) problems.

Example usage:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat With Crashes

Used to catch crash problems.

Example usage:

```
adb logcat -b crash
```

---

135- Adb Logcat With Native Crashes

Used to catch native (C/C++/Rust) crash problems.

Example usage:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat With Tombstone Analysis

Used to analyse native crash records.

Example usage:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat With Systrace Analysis

Used to analyse system trace records.

Example usage:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat With Perfetto Analysis

The modern system tracing tool, more thorough than logcat.

Example usage:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat With Atrace Analysis

Used for system-level tracing; the older form of Perfetto.

Example usage:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat With Traceview Analysis

Used for method-level profiling.

Example usage:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat With Method Tracing

Used to trace a given method.

Example usage:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat With A Heap Dump

Used to take a memory dump. Very useful in memory analysis.

Example usage:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat With The GC Log

Used to watch garbage collection events.

Example usage:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat With JNI Debugging

Used to catch JNI (Java Native Interface) problems.

Example usage:

```
adb logcat -s JNI
```

---

145- Adb Logcat With ART Debugging

Used to catch ART (Android Runtime) problems.

Example usage:

```
adb logcat -s Art
```

---

146- Adb Logcat With Dalvik Debugging

Used to catch Dalvik virtual machine problems. Used on older Android versions.

Example usage:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat And The Chatty Problem

When logcat writes too many logs it gives a "chatty" warning and skips lines. Fix it with a filter or a larger buffer.

Example usage (grow the buffer):

```
adb logcat -G 16M
```

Example usage (use a filter):

```
adb logcat -s MyApp
```

---

148- Adb Logcat With Coloured Output

In long log streams coloured output is easy on the eyes and makes the error visible straight away.

Example usage:

```
adb logcat -v color
```

Example usage (with time):

```
adb logcat -v time,color
```

---

149- Adb Logcat With A Bash Script

Used to write a script that collects logs automatically. A bug reporting system can be built on it.

Example script:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat Summary

As this list shows, logcat is not a single command but a set of commands. Combined with filtering, formats, buffers, files and other tools it becomes the strongest weapon in debugging. Once you know which combination fits which situation, finding a bug takes minutes. Remember: the first rule of a debugging session is always the same; clear with adb logcat -c first, then reproduce the bug, then watch with a filtered logcat.

@@ Android Debug Bridge Connection
@: ADB Wireless Connection Commands
1- Turning On Wireless Debugging

Developer options must be turned on on the device first. To do that, tap Settings → About phone → Build number 7 times. Then turn on Settings → System → Developer options → Wireless debugging. Without this step none of the adb pair and adb connect commands work.

Example path:

```
Settings → About phone → Build number (tap 7 times)
Settings → System → Developer options → Wireless debugging (on)
```

---

2- Finding The Device's IP Address And Port

Once wireless debugging is on, open the "Pair device with pairing code" option. The device's IP address and a random port number appear there, for example 192.168.1.42:37123. This port is only for pairing and changes afterwards. A 6-digit pairing code is written on the same screen too.

Example view:

```
IP address & Port
192.168.1.42:37123

Wi-Fi pairing code
482915
```

---

3- The Adb Pair Command

This command pairs the device. It is run on a computer or in Termux. Type the IP address and pairing port shown on the device. When the command runs, the terminal asks Enter pairing code: and waits for the 6-digit code. When the right code is entered, the Successfully paired message appears.

Example usage:

```
adb pair 192.168.1.42:37123
```

Example output:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

If a wrong code is entered, it fails like this:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Other Ways To Give The Code To Adb Pair

The code does not have to be typed by hand every time. Some versions accept it directly as a parameter, although for security reasons this is usually not supported. It can still be tried.

Example usage (on one line):

```
adb pair 192.168.1.42:37123 482915
```

Example usage (with a pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

These methods differ from version to version. The most reliable way is to run the command and type the code by hand.

---

5- What To Watch For With Adb Pair

The pairing port and the connection port are different. The pairing port is usually random and high, while the connection port is usually 5555. Pairing is done only once; adb connect is enough to connect to the same device again. But if you reset the device to factory settings or turn wireless debugging off, the pairing is reset and adb pair is needed again.

Example comparison:

```
Pairing:    adb pair 192.168.1.42:37123   → asks for a code, done once
Connecting: adb connect 192.168.1.42:5555  → no code, done every time
```

---

6- The Adb Connect Command

This command connects to a paired device. Type the IP address and port shown on the device's main wireless debugging screen. This port is usually 5555 but can sometimes be different. If the connection succeeds, the Connected to message appears.

Example usage:

```
adb connect 192.168.1.42:5555
```

Example output:

```
Connected to 192.168.1.42:5555
```

If it is already connected, it warns like this:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Without A Port

In some versions 5555 is tried by default when no port is given. But this behaviour is not the same on every device, so writing the port explicitly is the safest.

Example usage (default port):

```
adb connect 192.168.1.42
```

Example usage (explicit port):

```
adb connect 192.168.1.42:5555
```

---

8- Checking The Connection

After connecting, the adb devices command shows whether the device is really connected. The device's IP address should appear in the list with device next to it. If it says offline, the connection was made but has not settled; try again shortly.

Example usage:

```
adb devices
```

Example output:

```
List of devices attached
192.168.1.42:5555   device
```

Example output (not connected):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- The Adb Disconnect Command

Used to end the connection. The adb connection ends even if the device is still on the same network. To connect again, run adb connect again; no pairing is needed.

Example usage (one device):

```
adb disconnect 192.168.1.42:5555
```

Example output:

```
Disconnected 192.168.1.42:5555
```

Example usage (every device):

```
adb disconnect
```

Example output:

```
Disconnected everything
```

---

10- The Adb Tcpip Command

Used to switch a device connected over USB to wireless mode. This command makes the device listen on the given port, usually 5555. This method is more common on versions older than Android 11, because those versions have no adb pair.

Example usage:

```
adb tcpip 5555
```

Example output:

```
Restarting in TCP mode port: 5555
```

Then unplug the USB cable and run this command:

```
adb connect 192.168.1.42:5555
```

---

11- The Adb Usb Command

Used to go back from wireless mode to USB mode, when you want to connect the device with a cable again.

Example usage:

```
adb usb
```

Example output:

```
Restarting in USB mode
```

---

12- Adb Kill-Server And Start-Server

The adb server sometimes hangs or refuses connections. Then the server has to be stopped and started again. This resets every connection and gives a clean start.

Example usage:

```
adb kill-server
adb start-server
```

Example output:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Then connect again:

```
adb connect 192.168.1.42:5555
```

---

13- Being On The Same Network

For wireless adb the device and the computer or phone must be on the same Wi-Fi network. On different networks no connection is made. A guest network, the 5 GHz and 2.4 GHz split, a VPN or a firewall sometimes cause trouble. The simple fix: connect both devices to the same router and turn VPNs off.

Example check:

```
Device: Settings → Wi-Fi → Connected network (e.g. Home_Wifi)
Termux: Settings → Wi-Fi → Connected network (e.g. Home_Wifi)
```

---

14- Finding The IP Address By Hand

To find the device's IP address, follow Settings → Wi-Fi → Connected network → Network details. Or look in Termux with the command below. But the IP address can change over time, so checking the wireless debugging screen every time is the most reliable.

Example usage (Termux):

```
ifconfig wlan0
```

Example usage (on the device):

```
Settings → Wi-Fi → Connected network → IP address
```

---

15- Connecting To Itself From Termux On The Same Device

If Termux runs on the same device, adb is already local, but you can still connect over localhost with adb connect. This is usually done with 127.0.0.1:5555. Some devices refuse this; then the device's own LAN IP is used.

Example usage (localhost):

```
adb connect 127.0.0.1:5555
```

Example usage (its own LAN IP):

```
adb connect 192.168.1.42:5555
```

---

16- Running Commands With Several Devices Connected

When several devices are connected at once, choose the device with the -s parameter in every command. Otherwise adb fails with error: more than one device.

Example usage:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- When The Connection Drops

If the Wi-Fi signal weakens, the device sleeps or the network changes, the connection drops. The device then disappears from the adb devices list or shows as offline. The fix: run adb connect again. The pairing is usually still valid, so adb pair is not needed.

Example recovery:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- After The Device Restarts

When the device restarts, wireless debugging usually turns off. On some devices it stays on but the pairing is reset. After the device starts, go to the wireless debugging screen, take the new port and code, run adb pair again, then adb connect.

Example flow:

```
Device restarted → turn on Wireless debugging
New port: 42157, new code: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Common Errors While Pairing

Failed: Wrong password → the code was typed wrongly.
Failed to connect: Connection refused → the port is wrong or the device is not listening.
Failed: Pairing rejected → the device rejected the pairing; renew the code.
Timeout → there is a network problem; check that you are on the same network.

Example error and fix:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Fix: the code on the device has changed; take the new code and try again.

---

20- Turning Wireless Adb Off

Turning wireless debugging off when you are done is a good idea for security. You can turn it off in Settings or switch to USB mode with adb.

Example usage:

```
adb usb
adb disconnect
```

Example usage (from Settings):

```
Settings → System → Developer options → Wireless debugging (off)
```

---

21- Differences By Android Version

Android 11 and later: pairing with adb pair is required and a random port is used.
Android 10 and earlier: switch to wireless mode with adb tcpip 5555; there is no pairing.

Example comparison:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Security Notes

While wireless debugging is on, someone with bad intentions on the same network could try to reach the device. So keep this feature off on Wi-Fi networks you do not trust. The pairing code exists for security, but it is still wise to be careful. Turn it off when you are done.

Example precaution:

```
When not in use: Settings → Developer options → Wireless debugging (off)
On a safe network: turn it on only at home or at the office
```

---

23- The Whole Flow In One Place

The script below shows every step from scratch to connected. Just change the IP and port to your own device's.

Example flow:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Expected result:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- A Shortcut Script For A Lasting Connection

If you connect all the time, you can write a script and connect with a single command every time. For example you can add a function to the ~/.bashrc file in Termux.

Example function:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Enter the IP:port and the code shown on the device:"
    read -p "Address: " ADRES
    adb pair "$ADRES"
    read -p "Connection port (e.g. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Usage:

```
wifi_adb
```

---

25- Final Note

Wireless adb makes debugging possible without a USB cable and is especially handy when working in Termux on the phone itself. The two basic commands are adb pair and adb connect; the first pairs once, the second is used for every connection. The rest (adb devices, adb disconnect, adb tcpip, adb kill-server) exist to support those two commands and to fix problems. Do not forget the same-network rule, turn it off when not in use, and restart the server if trouble appears. That is all.
"#;

const TURKISH: &str = r#"@@ Logcat Komutları
1- Adb Logcat

Cihazdaki Tüm Günlükleri Canlı Olarak Terminale Akıtır. Komut Çalıştırıldığı Anda İtibaren Sistem, Uygulamalar, Servisler Ve Çekirdek Tarafından Üretilen Her Günlük Satırı Ekrana Düşer. Durdurmak İçin Klavyeden Ctrl + C Tuşlarına Basılır. Bu Komut Hata Ayıklamanın Temelidir; Ancak Filtresiz Kullanıldığında Saniyede Yüzlerce Satır Akabilir Ve İşine Yarayanı Bulmak Zorlaşır.

Örnek Kullanım:

```
adb logcat
```

Örnek Çıktı:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Mevcut Birikmiş Günlükleri Ekrana Döker Ve Komut Sonlanır. Yani Sürekli Akan Bir Akış Yerine O Anki Fotoğrafı Alır. Bir Hata Raporu Çıkarırken, Bir Çökme Sonrası Durumu Kaydederken Veya Bir Betiğin (Script) İçinden Tek Seferlik Günlük Almak İstediğinde Kullanılır.

Örnek Kullanım:

```
adb logcat -d
```

Örnek Kullanım (Dosyaya Kaydetme):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Bu Komut Özellikle Sunucuya Otomatik Rapor Gönderen Betiklerde Çok Kullanılır Çünkü Komut Kendiliğinden Biter Ve Betiği Kilitlemez.

---

3- Adb Logcat -C

Tüm Günlük Arabelleklerini Temizler. Yani O Ana Kadar Birikmiş Ne Varsa Siler. Yeni Bir Hata Ayıklama Oturumuna Başlamadan Önce Çalıştırıldığında, Sadece O An Itibarıyla Üretilen Günlükleri Görürsün Ve Eski Gürültü Seni Yanıltmaz. Bir Hatayı Yeniden Üretmeye Çalışırken İlk Adım Genellikle Budur.

Örnek Kullanım:

```
adb logcat -c
adb logcat
```

Yukarıdaki İki Komut Sırayla Çalıştırıldığında Tertemiz Bir Günlük Akışı Elde Edilir. Sadece Temizledikten Sonra Olan Olaylar Ekrana Düşer.

---

4- Adb Logcat -G

Mevcut Günlük Arabelleklerinin Boyutunu Ve Doluluk Oranını Gösterir. Hangi Arabelleğin Ne Kadar Dolu Olduğunu, Toplam Kapasitesini Ve Kullanılan Miktarını Satır Satır Listeler. Günlükler Çok Hızlı Akıyorsa Ve Eski Kayıtlar Siliniyorsa (Bu Duruma "Chatty" Denir), Önce Bu Komutla Duruma Bakılır.

Örnek Kullanım:

```
adb logcat -g
```

Örnek Çıktı:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Boyut

Günlük Arabelleğinin Boyutunu Değiştirir. Örneğin 256 Kib Olan Tamponu 16 Mib Yapabilirsin. Bu Sayede Daha Fazla Günlük Saklanır Ve Eski Kayıtlar Daha Geç Silinir. Uzun Süreli Bir Test Sırasında Veya Çok Gürültülü Bir Uygulamayı Ayıklarken Çok İşe Yarar. Cihazı Yeniden Başlatınca Bu Ayar Sıfırlanır.

Örnek Kullanım:

```
adb logcat -G 16M
```

Örnek Kullanım (Küçültmek İçin):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Dosya

Günlükleri Terminale Basmak Yerine Belirtilen Dosyaya Yazar. Bu Sayede Terminal Ekranı Kirlenmez Ve Kayıt Kalıcı Olur. Dosya Yolu Cihaz Üzerindedir, Bilgisayar Değil. Mobil Terminalde (Termux) Çalışırken Genellikle /Sdcard/ Altına Yazılır.

Örnek Kullanım:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Örnek Kullanım (Dosya Üzerine Ekleyerek Yazma):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

Yukarıdaki Örnekte Dosya 4 Mib'e Ulaştığında Yenisi Açılır Ve Toplam 10 Dosya Saklanır.

---

7- Adb Logcat -B Main

Sadece Ana Uygulama Arabelleğini İzler. Uygulamaların Log.D, Log.I, Log.E Gibi Çağrılarla Yazdığı Günlükler Buradadır. Varsayılan Olarak Zaten İzlenir Ama Sadece Bu Arabelleği İstemek İçin Açıkça Belirtilir. Uygulama Geliştirirken En Çok Bakılan Yerdir.

Örnek Kullanım:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Sistem Servislerinin Günlüklerini İzler. ActivityManager, WindowManager, PackageManager, PowerManager Gibi Servislerin Yazdığı Günlükler Buradadır. Uygulama Başlatma, İzin Verme, Servis Bağlama Gibi Olayları Görmek İçin Kullanılır. Genellikle Main İle Birlikte İzlenir.

Örnek Kullanım:

```
adb logcat -b system
```

Örnek Çıktı:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Sadece Çökme Günlüklerini İzler. Bir Uygulama Force Close Verdiğinde Yazılan FATAL EXCEPTION Blokları Buradadır. Bir Çökmeyi Ayıklarken Önce Bu Arabelleğe Bakmak En Hızlı Yoldur Çünkü Yığın İzi (Stack Trace) Direkt Buradadır.

Örnek Kullanım:

```
adb logcat -b crash
```

Örnek Çıktı:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Telsiz, Telefon, Sinyal Ve Veri Bağlantısı İle İlgili Günlükleri İzler. Sim Kart, Şebeke Kaydı, 4G/5G Bağlantısı, Sms, Arama Gibi Konularla Uğraşırken Kullanılır. Normal Uygulama Geliştirirken Genellikle Gerekmez.

Örnek Kullanım:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Sistem Olaylarının İkili (Binary) Günlüklerini İzler. Bu Arabellek Normal Metin Değil, Log.Event Çağrılarının Bıraktığı Kısa Kodlanmış Kayıtları İçerir. Am_Proc_Start, Am_Activity_Launch Gibi Etiketlerle Görünür. Performans Analizinde Kullanılır.

Örnek Kullanım:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Çekirdek (Kernel) Günlüklerini İzler. Linux Çekirdeğinin Yazdığı Mesajları Gösterir; Bu Genellikle Dmesg Komutunun Verdiği Çıktıya Benzer. Sürücü Hataları, Bellek Sorunları, Isı Ve Güç Yönetimi İle İlgili Bilgiler Buradadır. Root Gerektirebilir.

Örnek Kullanım:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Tüm Arabellekleri Tek Seferde İzler. Main, System, Crash, Radio, Events Ve Kernel Hepsi Karışık Olarak Akar. Her Şeyi Görmek İstediğinde Kullanılır Ama Çıktı Çok Kalabalık Olur, Bu Yüzden Genellikle Filtre İle Birlikte Kullanılır.

Örnek Kullanım:

```
adb logcat -b all
```

Örnek Kullanım (Filtreli):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Birden Fazla Arabellek

Virgülle Ayrılmış Birden Fazla Arabelleği Aynı Anda İzler. Örneğin Hem Uygulama Hem Çökme Günlüklerini Birlikte Görmek İçin Kullanılır. Sadece İhtiyacın Olan Kanalları Seçerek Çıktıyı Daraltabilirsin.

Örnek Kullanım:

```
adb logcat -b main,crash
```

Örnek Kullanım (Üç Kanal):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Varsayılan Kısa Çıktı Formatıdır. Her Satır Sadece Öncelik Seviyesi Ve Etiket İle Başlar. Hızlı Bakmak İçin İyidir Ama Zaman Damgası Olmadığı İçin Olayları Sıralamak Zorlaşır.

Örnek Kullanım:

```
adb logcat -v brief
```

Örnek Çıktı:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Tarih Ve Saat Damgası Ekler. Kısa Formata Göre Çok Daha Kullanışlıdır Çünkü Bir Olayın Ne Zaman Olduğunu Görürsün. Saniyenin Altında Milisaniye Değeri Vardır.

Örnek Kullanım:

```
adb logcat -v time
```

Örnek Çıktı:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

En Çok Önerilen Çıktı Formatıdır. Tarih, Saat, Öncelik, Etiket, Pid (İşlem Kimliği) Ve Tid (İş Parçacığı Kimliği) Bilgilerini İçerir. Çok İş Parçacıklı (Multithreaded) Uygulamalarda Hangi İş Parçacığının Ne Yaptığını Ayırt Etmek İçin Şarttır. Android Studio'nun Varsayılan Olarak Kullandığı Format Da Budur.

Örnek Kullanım:

```
adb logcat -v threadtime
```

Örnek Çıktı:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Tüm Meta Veri Alanlarını Ve Aralarında Boş Satır Bırakarak Ayrıntılı Çıktı Verir. Uzun Oturumlarda Her Günlüğü Bir Blok Olarak Görmek İçin Kullanılır Ama Yer Kaplar.

Örnek Kullanım:

```
adb logcat -v long
```

Örnek Çıktı:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Günlükleri Öncelik Seviyesine Göre Renklendirir. Hatalar Kırmızı, Uyarılar Turuncu, Bilgiler Yeşil, Hata Ayıklama Mavi Olur. Uzun Bir Akışta Gözünün Hemen Hataya Gitmesi İçin Çok Kullanışlıdır. Terminalin Renk Desteği Olması Gerekir.

Örnek Kullanım:

```
adb logcat -v color
```

Örnek Kullanım (Time İle Birlikte):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Time Formatına Yıl Bilgisini Ekler. Uzun Süreli Kayıtlarda Veya Birden Fazla Yıla Yayılan Arşivlerde Kullanılır. Normal Geliştirmede Genellikle Gerekmez.

Örnek Kullanım:

```
adb logcat -v year
```

Örnek Çıktı:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Günlüğü Üreten Uygulamanın Kullanıcı Kimliğini (Uid) Gösterir. Aynı Etiketi Birden Fazla Uygulama Kullanıyorsa Hangisinin Yazdığını Ayırt Etmek İçin Kullanılır. Modifiye Edici Bir Seçenektir, Yani Diğer Formatlarla Birleştirilir.

Örnek Kullanım:

```
adb logcat -v uid
```

Örnek Çıktı:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Etiket Ve Öncelik Filtresi

Belirli Bir Etikete (Tag) Sahip Ve Belirli Bir Öncelik Seviyesinin Üzerindeki Günlükleri Gösterir. Öncelik Seviyeleri Şunlardır: V (Verbose) En Düşük, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) Ve S (Silent) En Yüksek. Bu Filtreleme Yöntemi Logcat'in En Güçlü Özelliğidir.

Örnek Kullanım (Sadece Hatalar):

```
adb logcat *:E
```

Örnek Kullanım (Uyarı Ve Üzeri):

```
adb logcat *:W
```

Örnek Kullanım (Beyaz Liste):

```
adb logcat ActivityManager:I MyApp:D *:S
```

Yukarıdaki Son Örnekte ActivityManager Etiketli Info Ve Üzerini, MyApp Etiketli Debug Ve Üzerini Gösterir; Diğer Tüm Etiketleri Susturur. Bazı Shell'lerde * Karakteri Özel Olduğu İçin Tırnak İçine Alınmalıdır:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Etiket

Silent Filtresinin Kısayoludur. Sadece Belirtilen Etiketi Gösterir, Diğer Her Şeyi Susturur. Uygulama Geliştirirken Kendi Etiketine Odaklanmak İçin En Pratik Yoldur.

Örnek Kullanım:

```
adb logcat -s MyApp
```

Örnek Kullanım (Birden Fazla Etiket):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

Yukarıdaki Örnekte Sadece Üç Etiketin Günlükleri Akar, Diğer Her Şey Sessiz Kalır.

---

24- Adb Logcat -S İle Seviye Birlikte

Hem Etiketi Hem Öncelik Seviyesini Birlikte Belirtmek İçin Kullanılır. Örneğin Sadece MyApp Etiketinin Uyarı Ve Üzerini Görmek İçin.

Örnek Kullanım:

```
adb logcat MyApp:W *:S
```

Bu Komut Sadece MyApp Etiketli Warning, Error Ve Fatal Günlüklerini Gösterir.

---

25- Adb Logcat --Pid Filtresi

Sadece Belirli Bir İşlem Kimliğine (Pid) Ait Günlükleri Gösterir. Bir Uygulamanın Pid'sini Öğrenmek İçin Önce Adb Shell Pidof Kullanılır, Sonra Bu Pid İle Filtreleme Yapılır. Belirli Bir Oturumun Tüm Günlüklerini İzole Etmek İçin Mükemmeldir.

Örnek Kullanım (Pid Alma):

```
adb shell pidof com.example.app
```

Örnek Çıktı:

```
5678
```

Örnek Kullanım (Filtreleme):

```
adb logcat --pid=5678
```

Örnek Kullanım (Tek Satırda):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Zaman

Belirtilen Zaman Damgasından İtibaren Günlükleri Gösterir. Yani "Şu Andan Sonrasını Getir" Demek İçin Kullanılır. Büyük T Harfi İle Yazılır, Küçük T Farklı Bir Anlama Gelir.

Örnek Kullanım:

```
adb logcat -T "09-21 14:00:00.000"
```

Örnek Kullanım (Bugünden Itibaren):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Satır Sayısı

Küçük T Harfi İle Kullanıldığında, En Son Belirtilen Sayıda Satırı Gösterir Ve Komut Sonlanır. Yani -D Gibi Davranır Ama Sadece Son N Satırı Verir. Hızlı Bir Bakış İçin İdealdir.

Örnek Kullanım (Son 100 Satır):

```
adb logcat -t 100
```

Örnek Kullanım (Son 500 Satır Ve Filtre):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Düzenli İfade (Regex) İle İçerik Arama Yapar. İçeriğinde Belirtilen Kalıba Uyan Satırları Gösterir. Metin Tabanlı Hızlı Arama İçin Çok Güçlüdür; Özellikle Birden Fazla Kelimeyi Aynı Anda Aradığında Filtreleme Yapmaktan Daha Hızlıdır.

Örnek Kullanım (Tek Kelime):

```
adb logcat -e "Login"
```

Örnek Kullanım (Birden Fazla Kelime):

```
adb logcat -e "Login|Error|Crash"
```

Örnek Kullanım (Paket Adı İle):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Binary Çıktı

Günlükleri İkili (Binary) Formatta Çıktılar. Bu Format İnsan Tarafından Okunamaz Ama Başka Bir Logcat Tarafından Tekrar Okunabilir Veya Başka Araçlarla İşlenebilir. Otomatik Analiz Sistemlerinde Kullanılır.

Örnek Kullanım:

```
adb logcat -B > Ham_Log.Bin
```

Daha Sonra Bu Dosya Şöyle Okunur:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S İstatistik

Günlüklerin İstatistiklerini Gösterir. Hangi Etiketlerin Ne Kadar Günlük Ürettiğini, Hangi Etiketlerin "Chatty" Uyarısı Verdiğini Gösterir. Bir Uygulamanın Günlükleri "Spam"Lediğini Düşünüyorsan Bu Komutla Doğrulayabilirsin.

Örnek Kullanım:

```
adb logcat -S
```

Örnek Çıktı:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R Ve -N Birlikte Kullanım

Döngüsel (Rotating) Kayıt İçin Kullanılır. -R Her Dosyanın Boyutunu, -N Kaç Dosya Saklanacağını Belirler. Yani Bir Uygulama Uzun Süre Çalışacaksa Ve Sürekli Log Tutulmak İsteniyorsa Kullanılır. Eski Dosyalar Silinir, Yeni Dosyalar Açılır.

Örnek Kullanım:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

Yukarıdaki Örnekte Her Dosya 4 Mib Olduğunda Yenisi Açılır Ve Toplam 10 Dosya Saklanır. Böylece Disk Sonsuz Dolmaz.

---

32- Adb Logcat -P Filtre Betiği

Logcat İçin Özel Bir Filtre Betiği (Filter Script) Belirtir. Bu, Android'in Kendi Filtre Formatıdır Ve Genellikle Sistem Geliştiricileri Tarafından Kullanılır. Normal Uygulama Geliştiricileri Çok Nadir Kullanır.

Örnek Kullanım:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Zaman Damgaları

Bazı Formatlarda Günlük Satırlarının Başına Zaman Damgası Eklemek İçin Kullanılan Bir Seçenektir. Bazı Android Sürümlerinde Davranışı Değişir. Genellikle -V Time İle Aynı İşi Görür, Ancak Farklı Bir Biçimde.

Örnek Kullanım:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Her Satırı Printf Formatında Gösterir. Yani Öncelik (Pid) Etiket: Mesaj Şeklinde. Bazı Eski Araçlarla Uyumluluk İçin Kullanılır Ama Modern Android'de Çok Gerekmez.

Örnek Kullanım:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Sadece Ham Mesajı Gösterir, Hiçbir Meta Veri Eklemez. Etiket, Öncelik, Zaman Damgası Olmadan Sadece Günlüğün Kendisi Yazılır. Bir Çıktıyı Başka Bir Programa Aktarırken Kullanılır.

Örnek Kullanım:

```
adb logcat -v raw
```

Örnek Çıktı:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Her Satırın Başına Sadece Etiketi Yazıp Gerisini Getirir. Öncelik Seviyesi Ve Pid Görünmez. Belirli Bir Etikete Odaklanmışken Kullanışlıdır.

Örnek Kullanım:

```
adb logcat -v tag
```

Örnek Çıktı:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Her Satırın Başına Önce Pid Sonra Etiket Yazılır. Süreç Bazlı Analiz Yaparken Kullanışlıdır. Hangi Sürecin Ne Yazdığını Hızlıca Görmek İçin İyidir.

Örnek Kullanım:

```
adb logcat -v process
```

Örnek Çıktı:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Her Satırın Başına Sadece İş Parçacığı Kimliğini (Tid) Yazıp Gerisini Getirir. Çok İş Parçacıklı Analizlerde Hangi İş Parçacığının Aktif Olduğunu Görmek İçin Kullanılır. Kısa Bir Formattır.

Örnek Kullanım:

```
adb logcat -v thread
```

Örnek Çıktı:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D Ve -T Birlikte

Bir Zaman Aralığındaki Günlükleri Almak İçin Kullanılır. Örneğin Son Bir Saatteki Günlükleri Alıp Dosyaya Kaydetmek İçin.

Örnek Kullanım:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Örnek Kullanım (Dosyaya):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C Ve -D Sırayla

Temizle Ve Sonra Al İşlemini Aynı Anda Yapan Bir Akıştır. Örneğin Bir Test Başlatmadan Önce Temizleyip, Test Bittiğinde Günlükleri Almak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Dosyaya Kaydedip Bilgisayara Çekme

Mobil Terminalde Uzun Süre Kayıt Tutmak İçin Kullanışlıdır. Önce Cihazda Bir Dosyaya Yazılır, Sonra Adb Pull İle Bilgisayara Çekilir.

Örnek Kullanım (Cihazda Kaydet):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Örnek Kullanım (Bilgisayara Çek):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Ve Grep Birlikte

Mobil Terminalde (Termux) Grep İle Hızlı Metin Arama Yapmak İçin Kullanılır. Regex Filtresinden Daha Esnek Olabilir Çünkü Grep'in Tüm Seçeneklerini Kullanabilirsin.

Örnek Kullanım:

```
adb logcat | grep "Hata"
```

Örnek Kullanım (Büyük Küçük Harf Duymaz):

```
adb logcat | grep -I "error"
```

Örnek Kullanım (Satır Numarası İle):

```
adb logcat | grep -N "Exception"
```

Örnek Kullanım (Birden Fazla Kelime):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Ve Tail Birlikte

Canlı Akışı Son Birkaç Satıra İndirmek İçin Kullanılır. Tüm Günlük Yerine Sadece Son Kısmı Görmek İstediğinde.

Örnek Kullanım:

```
adb logcat | tail -100
```

Örnek Kullanım (Sürekli Takip):

```
adb logcat | tail -F
```

---

44- Adb Logcat Ve Head Birlikte

İlk Birkaç Satırı Görmek İçin Kullanılır. Uzun Bir Kayıttan Hızlı Bir Bakış Almak İçin İyidir.

Örnek Kullanım:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Ve Wc Birlikte

Toplam Satır Sayısını Görmek İçin Kullanılır. Örneğin Bir Test Sırasında Kaç Günlük Üretildiğini Ölçmek İçin.

Örnek Kullanım:

```
adb logcat -d | wc -l
```

Örnek Kullanım (Sadece Hataları Say):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Ve Sort Birlikte

Günlükleri Alfabetik Veya Sayısal Sıralamak İçin Kullanılır. Genellikle Analiz Sırasında Kullanılır.

Örnek Kullanım:

```
adb logcat -d | sort
```

Örnek Kullanım (Tersten Sıralama):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Ve Uniq Birlikte

Tekrar Eden Satırları Kaldırmak İçin Kullanılır. Bir Uygulama Aynı Hatayı Sürekli Yazıyorsa Bu Komut Çıktıyı Sadeleştirir.

Örnek Kullanım:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

Yukarıdaki Örnek En Sık Tekrar Eden 20 Satırı Sayarak Gösterir. Hata Ayıklamada Çok Kullanışlıdır.

---

48- Adb Logcat Ve Awk Birlikte

Belirli Sütunları Almak Veya Metin Üzerinde İşlem Yapmak İçin Kullanılır. Threadtime Formatında Pid Sütununu Almak Gibi.

Örnek Kullanım (Sadece Pid Sütunu):

```
adb logcat -v threadtime | awk '{print $3}'
```

Örnek Kullanım (Sadece Etiket Ve Mesaj):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Ve Xargs Birlikte

Her Satır İçin Bir Komut Çalıştırmak İçin Kullanılır. Örneğin Her Hata Satırı İçin Bir Bildirim Göndermek Gibi. Genellikle Otomasyonda Kullanılır.

Örnek Kullanım:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Ve Sed Birlikte

Metin Değiştirme İçin Kullanılır. Çıktıdaki Belirli Kalıpları Değiştirmek İçin İdealdir.

Örnek Kullanım:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

Yukarıdaki Örnek Tüm MyApp Yerine Uygulama Yazar.

---

51- Adb Logcat Ve Tee Birlikte

Hem Ekranda Göster Hem Dosyaya Yaz İşlemi İçin Kullanılır. Canlı Takip İle Kayıt Aynı Anda Yapılır.

Örnek Kullanım:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Ve Watch Birlikte

Belirli Aralıklarla Günlük Almak İçin Kullanılır. Örneğin Her 5 Saniyede Bir Günlük Özetini Almak.

Örnek Kullanım:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Ve Timeout Birlikte

Belirli Bir Süre Sonra Logcat'i Durdurmak İçin Kullanılır. Otomatik Testlerde Çok Kullanışlıdır.

Örnek Kullanım:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Ve Alias Birlikte

Sık Kullandığın Komutları Kısaltmak İçin Kullanılır. Termux'ta .Bashrc Veya .Zshrc Dosyasına Eklenir.

Örnek Kullanım (Alias Tanımla):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Sonra Sadece Şöyle Çağırırsın:

```
lce
```

---

55- Adb Logcat Ve Çoklu Cihaz

Birden Fazla Cihaz Bağlıyken Belirli Bir Cihazı Hedeflemek İçin Kullanılır. Önce Adb Devices İle Cihaz Listelenir, Sonra -S İle Cihaz Seçilir.

Örnek Kullanım:

```
adb devices
```

Örnek Çıktı:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Örnek Kullanım (Belirli Cihaz):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Ve Wireless Debugging

Kablosuz Hata Ayıklama İçin Kullanılır. Cihaz Aynı Wi-Fi Ağında Olduğunda Kablosuz Bağlanılır. Android 11 Ve Üzeri Destekler.

Örnek Kullanım (Cihazda Wireless Debugging Aç, Sonra):

```
adb pair 192.168.1.42:37123
```

Örnek Kullanım (Bağlan):

```
adb connect 192.168.1.42:5555
```

Örnek Kullanım (Logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Ve Bugreport

Tüm Sistemin Detaylı Raporunu Almak İçin Kullanılır. Logcat'in Yanında Dumpsys, Dmesg, Top Gibi Birçok Araç Çalıştırılır Ve Tek Bir Zip İçinde Toplanır. Hata Ayıklamanın En Kapsamlı Yoludur.

Örnek Kullanım:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Örnek Kullanım (Bilgisayara Çek):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Ve Dumpsys Birlikte

Logcat'in Yanında Sistem Servislerinin O Anki Durumunu Görmek İçin Kullanılır. Örneğin ActivityManager'In O An Ne Yaptığını Görmek.

Örnek Kullanım:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Ve Anr Trace

Uygulama Yanıt Vermiyor (Anr) Hatası Aldığında Yığın İzini Almak İçin Kullanılır. /Data/Anr/Traces.Txt Dosyasında Anr Kayıtları Bulunur.

Örnek Kullanım:

```
adb shell cat /Data/Anr/Traces.Txt
```

Örnek Kullanım (Root Gerekir):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Ve Tombstone

Yerel (Native) Çökmeler İçin Kullanılır. /Data/Tombstones/ Klasöründe Her Çökme İçin Bir Dosya Oluşur. C++ Veya Rust Kodunda Bir Hata Olduğunda Buraya Bakılır.

Örnek Kullanım:

```
adb shell ls /Data/Tombstones/
```

Örnek Kullanım (Son Tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Ve Systrace

Sistem Genelinde Performans İzleme İçin Kullanılır. Sadece Günlük Değil, Çekirdek Seviyesinde İzleme Verisi Toplar. Perfetto İle Birlikte Kullanılır.

Örnek Kullanım:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Ve Layout Inspector

Uygulamanın Ekrandaki Yerleşimini Analiz Etmek İçin Kullanılır. Layout Inspector Ile Birlikte Kullanılır Ama Logcat İle De İlgili Günlükleri İzleyebilirsin.

Örnek Kullanım:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Ve Memory Profiler

Bellek Kullanımını İzlemek İçin Kullanılır. Memory Profiler Genellikle Android Studio'da Bulunur Ama Logcat'ten De Bellek İle İlgili Günlükleri Görebilirsin.

Örnek Kullanım:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Ve Cpu Profiler

Cpu Kullanımını İzlemek İçin Kullanılır. Basit Ölçüm İçin Top Komutu Da Kullanılabilir.

Örnek Kullanım:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Ve Network Profiler

Ağ Trafiğini İzlemek İçin Kullanılır. Okhttp, Retrofit Gibi Kütüphaneler Logcat'e Yazar Ama Tam Analiz İçin Network Profiler Gerekir.

Örnek Kullanım:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Ve Gpu Profiler

Gpu Kullanımını İzlemek İçin Kullanılır. Basit Ölçüm İçin Gfxinfo Komutu Da Kullanılabilir.

Örnek Kullanım:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Ve Battery Historian

Pil Kullanımını İzlemek İçin Kullanılır. Dumpsys Battery İle De Bakılabilir.

Örnek Kullanım:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Ve Wifi Diagnose

Wi-Fi Sorunlarını Ayıklamak İçin Kullanılır. Bağlantı Kopmaları, Zayıf Sinyal Gibi Durumlar İçin Bakılır.

Örnek Kullanım:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Ve Bluetooth Diagnose

Bluetooth Sorunlarını Ayıklamak İçin Kullanılır. Eşleşme Sorunları, Kopma Gibi Durumlarda Bakılır.

Örnek Kullanım:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Ve Sensor Diagnose

Sensör Sorunlarını Ayıklamak İçin Kullanılır. İvmeölçer, Jiroskop, Manyetometre Gibi Sensörler İçin.

Örnek Kullanım:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Ve Kamera Diagnose

Kamera Sorunlarını Ayıklamak İçin Kullanılır. Fotoğraf Çekme, Video Kayıt Gibi Durumlar İçin.

Örnek Kullanım:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Ve Ses Diagnose

Ses Sorunlarını Ayıklamak İçin Kullanılır. Ses Çalmama, Mikrofon Çalışmama Gibi Durumlar İçin.

Örnek Kullanım:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Ve Konum Diagnose

Konum Sorunlarını Ayıklamak İçin Kullanılır. Gps Sinyali Almama, Konum Güncellenmeme Gibi Durumlar İçin.

Örnek Kullanım:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Ve Bildirim Diagnose

Bildirim Sorunlarını Ayıklamak İçin Kullanılır. Bildirim Çıkmama, Yanlış Zamanlama Gibi Durumlar İçin.

Örnek Kullanım:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Ve İzin Diagnose

İzin Sorunlarını Ayıklamak İçin Kullanılır. İzin Verilmeme, Reddedilme Gibi Durumlar İçin.

Örnek Kullanım:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Ve Webview Diagnose

Webview Sorunlarını Ayıklamak İçin Kullanılır. Sayfa Yüklenmeme, JavaScript Hatası Gibi Durumlar İçin.

Örnek Kullanım:

```
adb logcat -s Chromium WebView
```

Örnek Kullanım (Daha Ayrıntılı):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Ve Okhttp Diagnose

Okhttp Sorunlarını Ayıklamak İçin Kullanılır. Ağ İstekleri, Yanıtlar, Hatalar İçin.

Örnek Kullanım:

```
adb logcat -s Okhttp
```

Örnek Kullanım (Tüm Seviyeler):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Ve Retrofit Diagnose

Retrofit Sorunlarını Ayıklamak İçin Kullanılır. Genellikle Okhttp'in Üzerine Kurulu Olduğu İçin Onun Günlüklerine Bakılır.

Örnek Kullanım:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Ve Glide Diagnose

Glide (Resim Yükleme Kütüphanesi) Sorunlarını Ayıklamak İçin Kullanılır. Resim Yüklenmeme, Bellek Sorunu Gibi Durumlar İçin.

Örnek Kullanım:

```
adb logcat -s Glide
```

---

80- Adb Logcat Ve Picasso Diagnose

Picasso (Resim Yükleme Kütüphanesi) Sorunlarını Ayıklamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Ve Room Diagnose

Room (Veritabanı Kütüphanesi) Sorunlarını Ayıklamak İçin Kullanılır. Sorgu Hataları, Şema Sorunları İçin.

Örnek Kullanım:

```
adb logcat -s Room
```

---

82- Adb Logcat Ve Firebase Diagnose

Firebase Sorunlarını Ayıklamak İçin Kullanılır. Crashlytics, Analytics, Messaging Gibi Modüller İçin.

Örnek Kullanım:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Ve Crashlytics Diagnose

Firebase Crashlytics Sorunlarını Ayıklamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Ve Analytics Diagnose

Analitik Sorunlarını Ayıklamak İçin Kullanılır. Olay Gönderilmeme, Toplu Gönderim Sorunları İçin.

Örnek Kullanım:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Ve Adb Kablosuz Debugging

Kablosuz Hata Ayıklama İçin Adb'yi Yeniden Başlatmak Gerekir Bazen. Önce USB İle Bağlan, Sonra Kablosuz'a Geç.

Örnek Kullanım:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Ve Cihazı Yeniden Başlatma

Cihazı Yeniden Başlatıp Günlükleri İzlemek İçin Kullanılır. Özellikle Açılış Sırasında Olan Hatalar İçin.

Örnek Kullanım (İki Terminal):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Ve Uygulama Yeniden Başlatma

Sadece Bir Uygulamayı Yeniden Başlatıp Günlükleri İzlemek İçin Kullanılır.

Örnek Kullanım:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Ve Uygulama Yükleme

Bir Apk'yi Yükleyip Günlükleri İzlemek İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Ve Uygulama Kaldırma

Bir Uygulamayı Kaldırıp Günlükleri İzlemek İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Ve Ekran Görüntüsü

Bir Hata Anında Ekran Görüntüsü Almak Ve Günlüğü Kaydetmek İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Ve Ekran Kaydı

Bir Hata Anında Ekran Kaydı Almak Ve Günlüğü Kaydetmek İçin Kullanılır. Android 4.4 Ve Üzeri Destekler.

Örnek Kullanım:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Ve Monkey Test

Rastgele Kullanıcı Etkileşimi Simüle Ederek Uygulama Test Etmek İçin Kullanılır. Günlüklerle Birlikte Çok Etkilidir.

Örnek Kullanım:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Ve Ui Automator Test

Ui Otomasyon Testleri İçin Kullanılır. Test Sırasında Günlükleri İzlemek İçin.

Örnek Kullanım:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Ve Espresso Test

Espresso Testleri İçin Kullanılır. Ui Otomasyonunun Bir Alt Kümesidir.

Örnek Kullanım:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Ve Junit Test

Birim Testleri İçin Kullanılır. Genellikle Bilgisayarda Çalışır Ama Cihazda Da Çalıştırılabilir.

Örnek Kullanım:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Ve Strictmode

Uygulama Performans Sorunlarını Yakalamak İçin Kullanılır. Ana İş Parçacığında Disk Veya Ağ İşlemi Yapıldığında Uyarı Verir.

Örnek Kullanım:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Ve Choreographer

Kare Atlama (Frame Skip) Sorunlarını Yakalamak İçin Kullanılır. Animasyon Takılmalarında Bakılır.

Örnek Kullanım:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Ve Gpu Overdraw

Aşırı Çizim (Overdraw) Sorunlarını Yakalamak İçin Kullanılır. Görsel Performans Analizinde Kullanılır.

Örnek Kullanım:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Ve Render Thread

Çizim İş Parçacığı Sorunlarını Yakalamak İçin Kullanılır. Modern Android'de Çizim İşlemleri Bu İş Parçacığında Yapılır.

Örnek Kullanım:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Ve Gpu Renderer

Gpu Çizim Sorunlarını Yakalamak İçin Kullanılır. Profil Gpu Rendering Açıldığında Bakılır.

Örnek Kullanım:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Ve TextView İşlemleri

Metin Görünümü İle İlgili Sorunları Yakalamak İçin Kullanılır. Font Sorunları, Ölçüm Hataları İçin.

Örnek Kullanım:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Ve Recyclerview İşlemleri

Liste Görünümü İle İlgili Sorunları Yakalamak İçin Kullanılır. Kaydırma Performansı, Görünüm Geri Dönüşümü İçin.

Örnek Kullanım:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Ve Viewpager İşlemleri

Sayfa Geçişleri İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Ve Fragment İşlemleri

Fragment Yaşam Döngüsü Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Ve Activity İşlemleri

Activity Yaşam Döngüsü Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Ve Servis İşlemleri

Arka Plan Servisleri İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Ve Workmanager İşlemleri

Workmanager İle İlgili Sorunları Yakalamak İçin Kullanılır. Arka Plan Görevleri İçin.

Örnek Kullanım:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Ve Alarmmanager İşlemleri

Alarmmanager İle İlgili Sorunları Yakalamak İçin Kullanılır. Zamanlanmış Görevler İçin.

Örnek Kullanım:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Ve Sharedpreferences İşlemleri

Sharedpreferences (Basit Veri Saklama) İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Ve Sqlite İşlemleri

Sqlite (Yerel Veritabanı) İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Ve Dosya İşlemleri

Dosya Okuma Ve Yazma İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Ve Ağ İşlemleri

Ağ Bağlantısı İle İlgili Sorunları Yakalamak İçin Kullanılır. Http, Tcp, Udp Gibi.

Örnek Kullanım:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Ve Ssl İşlemleri

Ssl Sertifika Sorunlarını Yakalamak İçin Kullanılır. Güvenli Bağlantı Hatalarında Bakılır.

Örnek Kullanım:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Ve Json İşlemleri

Json Ayrıştırma Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Ve Xml İşlemleri

Xml Ayrıştırma Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Ve Image İşlemleri

Resim Yükleme Ve İşleme Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Ve Media İşlemleri

Medya Çalma Ve Kayıt Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Ve Exoplayer İşlemleri

Exoplayer (Medya Oynatıcı Kütüphanesi) Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Ve Chromecast İşlemleri

Chromecast Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Ve Ads İşlemleri

Reklam Kütüphaneleri İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Ve Billing İşlemleri

Google Play Faturalandırma Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Ve Map İşlemleri

Harita Kütüphaneleri İle İlgili Sorunları Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Ve Auth İşlemleri

Kimlik Doğrulama Sorunlarını Yakalamak İçin Kullanılır. Google Sign-In, Firebase Auth Gibi.

Örnek Kullanım:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Ve Notification İşlemleri

Bildirim Gönderme Ve Alma Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Ve Deep Link İşlemleri

Derin Bağlantı (Deep Link) Sorunlarını Yakalamak İçin Kullanılır. Bir Url'den Uygulama Açma Gibi.

Örnek Kullanım:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Ve Intent İşlemleri

Intent Geçişleri İle İlgili Sorunları Yakalamak İçin Kullanılır. Uygulamalar Arası Geçişler İçin.

Örnek Kullanım:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Ve Broadcast İşlemleri

Broadcast Mesajları İle İlgili Sorunları Yakalamak İçin Kullanılır. Sistem Ve Uygulama Arası Mesajlar İçin.

Örnek Kullanım:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Ve Content Provider İşlemleri

Content Provider Sorunlarını Yakalamak İçin Kullanılır. Uygulamalar Arası Veri Paylaşımı İçin.

Örnek Kullanım:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Ve Job Scheduler İşlemleri

Job Scheduler Sorunlarını Yakalamak İçin Kullanılır. Zamanlanmış Arka Plan Görevleri İçin.

Örnek Kullanım:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Ve Doze Mode İşlemleri

Doze Mode (Uyku Modu) Sorunlarını Yakalamak İçin Kullanılır. Cihaz Uykuya Geçtiğinde Arka Plan İşlemleri İçin.

Örnek Kullanım:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Ve Battery Optimization İşlemleri

Pil Optimizasyonu Sorunlarını Yakalamak İçin Kullanılır. Arka Planda Çalışma Kısıtlamaları İçin.

Örnek Kullanım:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Ve Memory Leak İşlemleri

Bellek Sızıntısı Sorunlarını Yakalamak İçin Kullanılır. Leakcanary Gibi Araçlarla Birlikte Kullanılır.

Örnek Kullanım:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Ve Anr İşlemleri

Anr (Uygulama Yanıt Vermiyor) Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Ve Crash İşlemleri

Çökme Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -b crash
```

---

135- Adb Logcat Ve Native Crash İşlemleri

Yerel (C/C++/Rust) Çökme Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Ve Tombstone Analizi

Yerel Çökme Kayıtlarını Analiz Etmek İçin Kullanılır.

Örnek Kullanım:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Ve Systrace Analizi

Sistem İzleme Kayıtlarını Analiz Etmek İçin Kullanılır.

Örnek Kullanım:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Ve Perfetto Analizi

Modern Sistem İzleme Aracı. Logcat'ten Daha Kapsamlıdır.

Örnek Kullanım:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Ve Atrace Analizi

Sistem Seviyesinde İzleme İçin Kullanılır. Perfetto'nun Eski Hali.

Örnek Kullanım:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Ve Traceview Analizi

Metot Seviyesinde Profil Oluşturma İçin Kullanılır.

Örnek Kullanım:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Ve Method Tracing

Belirli Bir Metodun İzini Sürmek İçin Kullanılır.

Örnek Kullanım:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Ve Heap Dump

Bellek Dökümü Almak İçin Kullanılır. Bellek Analizinde Çok Kullanışlıdır.

Örnek Kullanım:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Ve Gc Log

Çöp Toplama (Garbage Collection) Olaylarını İzlemek İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Ve Jni Debug

Jni (Java Native Interface) Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s JNI
```

---

145- Adb Logcat Ve Art Debug

Art (Android Runtime) Sorunlarını Yakalamak İçin Kullanılır.

Örnek Kullanım:

```
adb logcat -s Art
```

---

146- Adb Logcat Ve Dalvikvm Debug

Dalvik Sanal Makinesi Sorunlarını Yakalamak İçin Kullanılır. Eski Android Sürümlerinde Kullanılır.

Örnek Kullanım:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat Ve Bloklama (Chatty) Sorunu

Logcat Çok Fazla Günlük Yazdığında "Chatty" Uyarısı Verir Ve Satırları Atlar. Bunu Çözmek İçin Filtre Kullanmak Veya Tampon Boyutunu Büyütmek Gerekir.

Örnek Kullanım (Tampon Büyüt):

```
adb logcat -G 16M
```

Örnek Kullanım (Filtre Kullan):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Ve Renkli Çıktı

Uzun Günlük Akışlarında Renkli Çıktı Gözü Yormaz Ve Hatayı Hemen Görmeyi Sağlar.

Örnek Kullanım:

```
adb logcat -v color
```

Örnek Kullanım (Time İle):

```
adb logcat -v time,color
```

---

149- Adb Logcat Ve Bash Betiği

Otomatik Günlük Toplama Betiği Yazmak İçin Kullanılır. Bir Hata Raporu Sistemi Kurulabilir.

Örnek Betik:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat Ve Özet

Bu Listede Gördüğün Gibi Logcat Tek Başına Bir Komut Değil, Bir Komut Setidir. Filtreleme, Format, Arabellek, Dosya Ve Diğer Araçlarla Birlikte Kullanıldığında Hata Ayıklamanın En Güçlü Silahı Haline Gelir. Hangi Durumda Hangi Kombinasyonu Kullanacağını Bilirsen, Bir Hatayı Bulmak Dakikalar Sürer. Unutma: Bir Hata Ayıklama Oturumunun İlk Kuralı Her Zaman Aynıdır; Önce Adb Logcat -C İle Temizle, Sonra Hatayı Yeniden Üret, Sonra Filtreli Logcat İle İzle.

@@ Android Debug Bridge Bağlanma
@: Adb Kablosuz Bağlanma Komutları
1- Kablosuz Hata Ayıklamayı Açma

Öncelikle Cihazda Geliştirici Seçenekleri Açılmalıdır. Bunun İçin Ayarlar → Telefon Hakkında → Yapı Numarası'na 7 Kez Dokunulur. Ardından Ayarlar → Sistem → Geliştirici Seçenekleri → Kablosuz Hata Ayıklama Açılır. Bu Adım Olmadan Adb Pair Ve Adb Connect Komutlarının Hiçbiri Çalışmaz.

Örnek Yol:

```
Ayarlar → Telefon Hakkında → Yapı Numarası (7 Kez Dokun)
Ayarlar → Sistem → Geliştirici Seçenekleri → Kablosuz Hata Ayıklama (Aç)
```

---

2- Cihazın Ip Adresini Ve Portunu Öğrenme

Kablosuz Hata Ayıklama Açıldıktan Sonra "Cihazı Eşleştir" Seçeneğine Girilir. Burada Cihazın Ip Adresi Ve Rastgele Bir Port Numarası Görünür. Örneğin 192.168.1.42:37123 Gibi. Bu Port Sadece Eşleştirme İçindir, Sonradan Değişir. Aynı Ekranda 6 Haneli Bir Eşleştirme Kodu Da Yazılıdır.

Örnek Görünüm:

```
Ip Adresi Ve Bağlantı Noktası
192.168.1.42:37123

Eşleştirme Kodu
482915
```

---

3- Adb Pair Komutu

Bu Komut Cihazı Eşleştirmek İçin Kullanılır. Bilgisayarda Veya Termux'ta Çalıştırılır. Cihazda Görünen Ip Adresi Ve Eşleştirme Portu Yazılır. Komut Çalıştırıldığında Terminal Enter Pairing Code: Diye Sorar Ve 6 Haneli Kod Beklenir. Doğru Kod Girildiğinde Successfully Paired Mesajı Çıkar.

Örnek Kullanım:

```
adb pair 192.168.1.42:37123
```

Örnek Çıktı:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Yanlış Kod Girilirse Şu Şekilde Hata Verir:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Adb Pair Komutunda Kod Girmenin Farklı Yolları

Kod Her Seferinde Elle Girilmek Zorunda Değildir. Bazı Sürümlerde Komuta Direkt Parametre Olarak Verilebilir Ama Genellikle Güvenlik Nedeniyle Desteklenmez. Yine De Denenebilir.

Örnek Kullanım (Tek Satırda):

```
adb pair 192.168.1.42:37123 482915
```

Örnek Kullanım (Boru İle):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Bu Yöntemler Cihaz Sürümüne Göre Değişir. En Güvenilir Yol Komutu Çalıştırıp Elle Girmektir.

---

5- Adb Pair Komutunda Dikkat Edilecekler

Eşleştirme Portu Ile Bağlantı Portu Farklıdır. Eşleştirme Portu Genellikle Rastgele Ve Yüksektir, Bağlantı Portu Ise Genellikle 5555'Tir. Eşleştirme Sadece Bir Kez Yapılır; Aynı Cihazla Tekrar Bağlanmak İçin Adb Connect Yeterlidir. Ancak Cihazı Fabrika Ayarlarına Döndürürsen Veya Kablosuz Hata Ayıklamayı Kapatırsan Eşleşme Sıfırlanır Ve Yeniden Adb Pair Gerekir.

Örnek Karşılaştırma:

```
Eşleştirme: adb pair 192.168.1.42:37123   → Kod İster, Bir Kez Yapılır
Bağlanma:  adb connect 192.168.1.42:5555  → Kod İstemez, Her Seferinde Yapılır
```

---

6- Adb Connect Komutu

Bu Komut Eşleştirilmiş Cihaza Bağlanmak İçin Kullanılır. Cihazda Ana Kablosuz Hata Ayıklama Ekranında Görünen Ip Adresi Ve Bağlantı Noktası Yazılır. Bu Port Genellikle 5555'Tir Ama Bazen Farklı Da Olabilir. Bağlantı Başarılıysa Connected To Mesajı Çıkar.

Örnek Kullanım:

```
adb connect 192.168.1.42:5555
```

Örnek Çıktı:

```
Connected to 192.168.1.42:5555
```

Zaten Bağlıysa Şu Şekilde Uyarır:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Komutunun Parametresiz Hali

Bazı Sürümlerde Port Belirtilmezse Varsayılan Olarak 5555 Denenir. Ancak Bu Davranış Her Cihazda Aynı Değildir, Bu Yüzden Portu Açıkça Yazmak En Güvenlisidir.

Örnek Kullanım (Varsayılan Port):

```
adb connect 192.168.1.42
```

Örnek Kullanım (Açık Port):

```
adb connect 192.168.1.42:5555
```

---

8- Bağlantıyı Doğrulama

Bağlantı Kurulduktan Sonra Cihazın Gerçekten Bağlı Olup Olmadığını Görmek İçin Adb Devices Komutu Kullanılır. Listede Cihazın Ip Adresi Ve Yanında Device Yazısı Görünmelidir. Offline Yazıyorsa Bağlantı Kurulmuş Ama Oturmamıştır, Kısa Süre Sonra Tekrar Denenmelidir.

Örnek Kullanım:

```
adb devices
```

Örnek Çıktı:

```
List of devices attached
192.168.1.42:5555   device
```

Örnek Çıktı (Bağlı Değil):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Adb Disconnect Komutu

Bağlantıyı Kesmek İçin Kullanılır. Cihaz Hâlâ Aynı Ağda Olsa Bile Adb Bağlantısı Sona Erer. Tekrar Bağlanmak İçin Yeniden Adb Connect Çalıştırılır, Eşleştirme Gerekmez.

Örnek Kullanım (Belirli Cihaz):

```
adb disconnect 192.168.1.42:5555
```

Örnek Çıktı:

```
Disconnected 192.168.1.42:5555
```

Örnek Kullanım (Tüm Cihazlar):

```
adb disconnect
```

Örnek Çıktı:

```
Disconnected everything
```

---

10- Adb Tcpip Komutu

Usb İle Bağlıyken Cihazı Kablosuz Moda Geçirmek İçin Kullanılır. Bu Komut Cihazın Belirtilen Portu Dinlemesini Sağlar. Genellikle 5555 Kullanılır. Bu Yöntem Android 11'Den Eski Sürümlerde Daha Yaygındır Çünkü O Sürümlerde Adb Pair Yoktur.

Örnek Kullanım:

```
adb tcpip 5555
```

Örnek Çıktı:

```
Restarting in TCP mode port: 5555
```

Sonra Usb Kablosu Çıkarılır Ve Şu Komut Çalıştırılır:

```
adb connect 192.168.1.42:5555
```

---

11- Adb Usb Komutu

Kablosuz Moddan Usb Moduna Geri Dönmek İçin Kullanılır. Cihaz Tekrar Kablo İle Bağlanmak İsteniyorsa Bu Komut Çalıştırılır.

Örnek Kullanım:

```
adb usb
```

Örnek Çıktı:

```
Restarting in USB mode
```

---

12- Adb Kill-Server Ve Start-Server

Adb Sunucusu Bazen Takılır Veya Bağlantıyı Kabul Etmez. Bu Durumda Sunucuyu Kapatıp Yeniden Başlatmak Gerekir. Bu İşlem Tüm Bağlantıları Sıfırlar Ve Temiz Bir Başlangıç Sağlar.

Örnek Kullanım:

```
adb kill-server
adb start-server
```

Örnek Çıktı:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Sonra Yeniden Bağlanılır:

```
adb connect 192.168.1.42:5555
```

---

13- Aynı Ağda Olma Şartı

Kablosuz Adb Için Cihaz İle Bilgisayarın Veya Telefonun Aynı Wi-Fi Ağında Olması Gerekir. Farklı Ağlardaysa Bağlantı Kurulmaz. Misafir Ağı, 5 Ghz Ve 2.4 Ghz Farkı, Vpn Veya Güvenlik Duvarı Bazen Sorun Çıkarır. Basit Çözüm: İki Cihazı Da Aynı Modeme Bağla, Vpn'leri Kapat.

Örnek Kontrol:

```
Cihaz: Ayarlar → Wi-Fi → Bağlı Ağ (Örn. Ev_Wifi)
Termux: Ayarlar → Wi-Fi → Bağlı Ağ (Örn. Ev_Wifi)
```

---

14- Ip Adresini Elle Öğrenme

Cihazın Ip Adresini Bulmak İçin Ayarlar → Wi-Fi → Bağlı Ağ → Ağ Detayları Yolunu Izleyebilirsin. Veya Termux'ta Şu Komutlarla Bakabilirsin. Ancak Bu Ip Adresi Zamanla Değişebilir, Bu Yüzden Her Seferinde Kablosuz Hata Ayıklama Ekranından Bakmak En Doğrusudur.

Örnek Kullanım (Termux):

```
ifconfig wlan0
```

Örnek Kullanım (Cihazda):

```
Ayarlar → Wi-Fi → Bağlı Ağ → Ip Adresi
```

---

15- Aynı Cihazda Termux İle Kendine Bağlanma

Eğer Termux Aynı Cihazda Çalışıyorsa Adb Zaten Yereldir, Ancak Yine De Adb Connect İle Localhost Üzerinden Bağlanılabilir. Bu Yöntem Genellikle 127.0.0.1:5555 İle Yapılır. Ancak Bazı Cihazlar Bunu Reddeder, O Zaman Cihazın Kendi Lan Ip'si Kullanılır.

Örnek Kullanım (Localhost):

```
adb connect 127.0.0.1:5555
```

Örnek Kullanım (Kendi Lan Ip'si):

```
adb connect 192.168.1.42:5555
```

---

16- Birden Fazla Cihaz Bağlıyken Komut Çalıştırma

Aynı Anda Birden Fazla Cihaz Bağlıysa Her Komutta -S Parametresi İle Cihaz Seçilir. Aksi Halde Adb Error: More Than One Device Hatası Verir.

Örnek Kullanım:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Bağlantı Koptuğunda Ne Olur

Wi-Fi Sinyali Zayıflar, Cihaz Uykuya Geçer Veya Ağ Değişirse Bağlantı Kopar. Bu Durumda Adb Devices Listesinde Cihaz Kaybolur Veya Offline Görünür. Çözüm: Yeniden Adb Connect Çalıştırmak. Eşleştirme Genellikle Hâlâ Geçerlidir, Adb Pair Gerekmez.

Örnek Kurtarma:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Cihaz Yeniden Başlatıldığında

Cihaz Yeniden Başlatıldığında Kablosuz Hata Ayıklama Genellikle Kapanır. Bazı Cihazlarda Açık Kalır Ama Eşleştirme Sıfırlanır. Cihaz Açıldıktan Sonra Kablosuz Hata Ayıklama Ekranına Git, Yeni Portu Ve Kodu Al, Yeniden Adb Pair Yap, Sonra Adb Connect Yap.

Örnek Akış:

```
Cihaz Yeniden Başladı → Kablosuz Hata Ayıklama Aç
Yeni Port: 42157, Yeni Kod: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Eşleştirme Sırasında Sık Karşılaşılan Hatalar

Failed: Wrong Password → Kod Yanlış Girilmiş.
Failed To Connect: Connection Refused → Port Yanlış Veya Cihaz Dinlemiyor.
Failed: Pairing Rejected → Cihazda Eşleştirme Reddedilmiş, Kodu Yenile.
Timeout → Ağ Sorunu Var, Aynı Ağda Mısınız Kontrol Et.

Örnek Hata Ve Çözüm:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Çözüm: Cihazdaki Kod Değişmiştir, Yeni Kodu Al Ve Tekrar Dene.

---

20- Kablosuz Adb'yi Kapatma

İşin Bittiğinde Kablosuz Hata Ayıklamayı Kapatmak Güvenlik Açısından Iyi Bir Fikirdir. Ayarlardan Kapatabilir Veya Adb İle Usb Moduna Geçebilirsin.

Örnek Kullanım:

```
adb usb
adb disconnect
```

Örnek Kullanım (Ayarlardan):

```
Ayarlar → Sistem → Geliştirici Seçenekleri → Kablosuz Hata Ayıklama (Kapat)
```

---

21- Android Sürümüne Göre Farklar

Android 11 Ve Üzeri: Adb Pair İle Eşleştirme Zorunludur, Rastgele Port Kullanılır.
Android 10 Ve Altı: Adb Tcpip 5555 İle Kablosuz Moda Geçilir, Eşleştirme Yoktur.

Örnek Karşılaştırma:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Güvenlik Notları

Kablosuz Hata Ayıklama Açıkken Aynı Ağdaki Kötü Niyetli Biri Cihaza Erişmeye Çalışabilir. Bu Yüzden Güvenmediğin Wi-Fi Ağlarında Bu Özelliği Kapalı Tut. Eşleştirme Kodu Zaten Güvenlik Için Vardır, Ancak Yine De Tedbirli Olmak Gerekir. Işin Bittiğinde Kapat.

Örnek Önlem:

```
Kullanmadığında: Ayarlar → Geliştirici Seçenekleri → Kablosuz Hata Ayıklama (Kapat)
Güvenli Ağda: Sadece Evde Veya Ofiste Aç
```

---

23- Tek Satırda Tüm Akış

Aşağıdaki Betik, Sıfırdan Bağlanmaya Kadar Tüm Adımları Gösterir. Ip Ve Portu Kendi Cihazına Göre Değiştirmen Yeterli.

Örnek Akış:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Beklenen Sonuç:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Kalıcı Bağlantı Için Kısayol Betiği

Sürekli Bağlanıyorsan Bir Betik Yazıp Her Seferinde Tek Komutla Bağlanabilirsin. Örneğin Termux'ta ~/.Bashrc Dosyasına Fonksiyon Ekleyebilirsin.

Örnek Fonksiyon:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Cihazdaki Ip:port Ve Kodu Gir:"
    read -p "Adres: " ADRES
    adb pair "$ADRES"
    read -p "Bağlantı Portu (Örn. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Kullanım:

```
wifi_adb
```

---

25- Son Not

Kablosuz Adb, Usb Kablo Olmadan Hata Ayıklamayı Sağlar Ve Özellikle Telefonun Kendisinde Termux İle Çalışırken Çok Kullanışlıdır. İki Temel Komut Adb Pair Ve Adb Connect'Tir; Birincisi Bir Kez Eşleştirme, İkincisi Her Bağlantıda Kullanılır. Geri Kalan Komutlar (Adb Devices, Adb Disconnect, Adb Tcpip, Adb Kill-Server) Ise Bu Iki Komutu Desteklemek Ve Sorun Gidermek Içindir. Aynı Ağda Olma Şartını Unutma, Kullanmadığında Kapat, Sorun Çıkarsa Sunucuyu Yeniden Başlat. Bu Kadar.
"#;

const GERMAN: &str = r#"@@ Logcat-Befehle
1- Adb Logcat

Leitet alle Protokolle des Geräts live ins Terminal. Ab dem Moment, in dem der Befehl läuft, erscheint jede Protokollzeile von System, Apps, Diensten und Kernel auf dem Bildschirm. Zum Beenden Strg + C auf der Tastatur drücken. Dieser Befehl ist die Grundlage der Fehlersuche; ohne Filter können jedoch Hunderte Zeilen pro Sekunde durchlaufen, und das Gesuchte ist schwer zu finden.

Beispiel:

```
adb logcat
```

Beispielausgabe:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Gibt die bereits gesammelten Protokolle aus, danach endet der Befehl. Statt eines fortlaufenden Stroms entsteht eine Momentaufnahme. Nützlich beim Erstellen eines Fehlerberichts, beim Sichern des Zustands nach einem Absturz oder wenn ein Skript einmalig Protokolle holen soll.

Beispiel:

```
adb logcat -d
```

Beispiel (in eine Datei speichern):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Dieser Befehl wird oft in Skripten verwendet, die automatisch Berichte an einen Server senden, weil er von selbst endet und das Skript nie blockiert.

---

3- Adb Logcat -C

Leert alle Protokollpuffer und löscht alles bisher Gesammelte. Wer ihn vor einer neuen Fehlersuche ausführt, sieht nur noch Protokolle ab diesem Moment, und altes Rauschen führt nicht in die Irre. Beim Nachstellen eines Fehlers ist das meist der erste Schritt.

Beispiel:

```
adb logcat -c
adb logcat
```

Führt man die beiden Befehle oben nacheinander aus, erhält man einen völlig sauberen Protokollstrom. Nur was nach dem Leeren geschieht, erscheint auf dem Bildschirm.

---

4- Adb Logcat -G

Zeigt die Größe der aktuellen Protokollpuffer und wie voll sie sind. Zeile für Zeile wird aufgeführt, wie voll jeder Puffer ist, wie groß er insgesamt ist und wie viel belegt ist. Laufen die Protokolle sehr schnell und werden alte Einträge verworfen (das nennt man „chatty“), schaut man zuerst hier nach.

Beispiel:

```
adb logcat -g
```

Beispielausgabe:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Größe

Ändert die Größe des Protokollpuffers. So lässt sich etwa ein Puffer von 256 KiB auf 16 MiB vergrößern. Dadurch werden mehr Protokolle aufbewahrt und alte Einträge später verworfen. Sehr hilfreich bei langen Tests oder beim Debuggen einer sehr gesprächigen App. Die Einstellung wird beim Neustart des Geräts zurückgesetzt.

Beispiel:

```
adb logcat -G 16M
```

Beispiel (zum Verkleinern):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Datei

Schreibt die Protokolle in die angegebene Datei, statt sie im Terminal auszugeben. Das Terminal bleibt sauber, und die Aufzeichnung bleibt erhalten. Der Dateipfad liegt auf dem Gerät, nicht auf einem Computer. In einem mobilen Terminal (Termux) wird meist unter /Sdcard/ geschrieben.

Beispiel:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Beispiel (an die Datei anhängen):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

Im Beispiel oben wird eine neue Datei geöffnet, sobald die Datei 4 MiB erreicht, und insgesamt werden 10 Dateien aufbewahrt.

---

7- Adb Logcat -B Main

Beobachtet nur den Hauptpuffer der Apps. Hier landen die Protokolle, die Apps mit Aufrufen wie Log.d, Log.i und Log.e schreiben. Er wird standardmäßig ohnehin beobachtet, man nennt ihn aber ausdrücklich, wenn man nur diesen Puffer möchte. Bei der App-Entwicklung schaut man hier am häufigsten hin.

Beispiel:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Beobachtet die Protokolle der Systemdienste. Hier landen die Protokolle von Diensten wie ActivityManager, WindowManager, PackageManager und PowerManager. Damit sieht man Ereignisse wie App-Starts, Berechtigungen und Dienstbindungen. Meist wird er zusammen mit main beobachtet.

Beispiel:

```
adb logcat -b system
```

Beispielausgabe:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Beobachtet nur Absturzprotokolle. Hier landen die FATAL EXCEPTION-Blöcke, die beim Zwangsbeenden einer App geschrieben werden. Beim Untersuchen eines Absturzes ist dieser Puffer der schnellste Weg, weil der Stacktrace direkt hier steht.

Beispiel:

```
adb logcat -b crash
```

Beispielausgabe:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Beobachtet Protokolle zu Funk, Telefonie, Signal und Datenverbindung. Nützlich bei SIM-Karte, Netzregistrierung, 4G/5G-Verbindung, SMS oder Anrufen. Für normale App-Entwicklung meist nicht nötig.

Beispiel:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Beobachtet die binären Protokolle von Systemereignissen. Dieser Puffer enthält keinen normalen Text, sondern kurze, codierte Einträge aus Log.event-Aufrufen. Sie erscheinen mit Tags wie am_proc_start und am_activity_launch. Wird bei der Leistungsanalyse verwendet.

Beispiel:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Beobachtet die Kernel-Protokolle. Zeigt die Meldungen des Linux-Kernels; das ähnelt meist der Ausgabe des Befehls dmesg. Hier stehen Treiberfehler, Speicherprobleme sowie Informationen zu Wärme und Energieverwaltung. Kann Root erfordern.

Beispiel:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Beobachtet alle Puffer auf einmal. Main, system, crash, radio, events und kernel laufen gemischt durch. Nützlich, wenn man alles sehen will, doch die Ausgabe wird sehr voll, daher meist mit einem Filter kombiniert.

Beispiel:

```
adb logcat -b all
```

Beispiel (gefiltert):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Mehrere Puffer

Beobachtet mehrere durch Kommas getrennte Puffer gleichzeitig, etwa um App- und Absturzprotokolle zusammen zu sehen. Wählt man nur die benötigten Kanäle, lässt sich die Ausgabe eingrenzen.

Beispiel:

```
adb logcat -b main,crash
```

Beispiel (drei Kanäle):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Das kurze Standardformat. Jede Zeile beginnt nur mit der Priorität und dem Tag. Gut für einen schnellen Blick, doch ohne Zeitstempel lassen sich Ereignisse schwer ordnen.

Beispiel:

```
adb logcat -v brief
```

Beispielausgabe:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Fügt einen Datums- und Zeitstempel hinzu. Deutlich nützlicher als das Kurzformat, weil man sieht, wann ein Ereignis geschah. Unter der Sekunde werden Millisekunden angezeigt.

Beispiel:

```
adb logcat -v time
```

Beispielausgabe:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

Das am meisten empfohlene Ausgabeformat. Es enthält Datum, Uhrzeit, Priorität, Tag, PID (Prozess-ID) und TID (Thread-ID). In Apps mit mehreren Threads ist es unverzichtbar, um zu erkennen, welcher Thread was getan hat. Android Studio verwendet es ebenfalls standardmäßig.

Beispiel:

```
adb logcat -v threadtime
```

Beispielausgabe:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Liefert eine ausführliche Ausgabe mit allen Metadatenfeldern und einer Leerzeile zwischen den Einträgen. In langen Sitzungen sieht man so jedes Protokoll als Block, es braucht aber Platz.

Beispiel:

```
adb logcat -v long
```

Beispielausgabe:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Färbt Protokolle nach Priorität ein. Fehler werden rot, Warnungen orange, Infos grün und Debug-Meldungen blau. In einem langen Strom springt das Auge sofort zum Fehler. Das Terminal muss Farben unterstützen.

Beispiel:

```
adb logcat -v color
```

Beispiel (zusammen mit time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Ergänzt das Zeitformat um das Jahr. Für lange Aufzeichnungen oder Archive über mehrere Jahre. In der normalen Entwicklung meist nicht nötig.

Beispiel:

```
adb logcat -v year
```

Beispielausgabe:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Zeigt die Benutzer-ID (UID) der App, die das Protokoll erzeugt hat. Hilfreich, wenn mehrere Apps dasselbe Tag verwenden. Es ist ein Modifikator und wird mit anderen Formaten kombiniert.

Beispiel:

```
adb logcat -v uid
```

Beispielausgabe:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Tag- Und Prioritätsfilter

Zeigt Protokolle mit einem bestimmten Tag ab einer bestimmten Priorität. Die Prioritäten sind: V (Verbose) die niedrigste, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) und S (Silent) die höchste. Diese Art zu filtern ist die stärkste Funktion von logcat.

Beispiel (nur Fehler):

```
adb logcat *:E
```

Beispiel (Warnungen und höher):

```
adb logcat *:W
```

Beispiel (Positivliste):

```
adb logcat ActivityManager:I MyApp:D *:S
```

Das letzte Beispiel zeigt ActivityManager ab Info und MyApp ab Debug und schaltet alle anderen Tags stumm. In manchen Shells ist das Zeichen * besonders und muss in Anführungszeichen stehen:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Tag

Eine Abkürzung für den Silent-Filter. Zeigt nur das angegebene Tag und schaltet alles andere stumm. Der praktischste Weg, sich bei der App-Entwicklung auf das eigene Tag zu konzentrieren.

Beispiel:

```
adb logcat -s MyApp
```

Beispiel (mehrere Tags):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

Im Beispiel oben laufen nur die Protokolle von drei Tags durch; alles andere bleibt stumm.

---

24- Adb Logcat -S Mit Priorität

Gibt Tag und Priorität gemeinsam an, etwa um vom Tag MyApp nur Warnungen und höher zu sehen.

Beispiel:

```
adb logcat MyApp:W *:S
```

Dieser Befehl zeigt nur Warning-, Error- und Fatal-Protokolle mit dem Tag MyApp.

---

25- Adb Logcat --Pid Filter

Zeigt nur die Protokolle einer bestimmten Prozess-ID (PID). Um die PID einer App zu erfahren, wird zuerst adb shell pidof verwendet und dann nach dieser PID gefiltert. Ideal, um alle Protokolle einer Sitzung zu isolieren.

Beispiel (PID ermitteln):

```
adb shell pidof com.example.app
```

Beispielausgabe:

```
5678
```

Beispiel (filtern):

```
adb logcat --pid=5678
```

Beispiel (in einer Zeile):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Zeit

Zeigt Protokolle ab dem angegebenen Zeitstempel, also „bring mir alles ab jetzt“. Es wird mit großem T geschrieben; ein kleines t bedeutet etwas anderes.

Beispiel:

```
adb logcat -T "09-21 14:00:00.000"
```

Beispiel (ab heute):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Zeilenanzahl

Mit kleinem t werden die angegebene Anzahl der letzten Zeilen gezeigt, danach endet der Befehl. Es verhält sich wie -d, liefert aber nur die letzten N Zeilen. Ideal für einen schnellen Blick.

Beispiel (letzte 100 Zeilen):

```
adb logcat -t 100
```

Beispiel (letzte 500 Zeilen und ein Filter):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Durchsucht den Inhalt mit einem regulären Ausdruck (Regex) und zeigt die passenden Zeilen. Sehr stark für schnelle Textsuche; vor allem bei mehreren Wörtern gleichzeitig ist es schneller als Filtern.

Beispiel (ein Wort):

```
adb logcat -e "Login"
```

Beispiel (mehrere Wörter):

```
adb logcat -e "Login|Error|Crash"
```

Beispiel (mit Paketnamen):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Binäre Ausgabe

Schreibt die Protokolle im Binärformat. Menschen können es nicht lesen, aber ein anderes logcat kann es wieder einlesen oder andere Werkzeuge können es verarbeiten. Wird in automatischen Analysesystemen genutzt.

Beispiel:

```
adb logcat -B > Ham_Log.Bin
```

Später wird diese Datei so gelesen:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Statistik

Zeigt Protokollstatistiken: wie viele Protokolle jedes Tag erzeugt hat und welche Tags die „chatty“-Warnung ausgelöst haben. Wer vermutet, dass eine App das Protokoll flutet, kann es damit bestätigen.

Beispiel:

```
adb logcat -S
```

Beispielausgabe:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R Und -N Zusammen

Für rotierende Protokolle. -r legt die Größe jeder Datei fest, -n wie viele Dateien aufbewahrt werden. Nützlich, wenn eine App lange läuft und ständig protokolliert werden soll. Alte Dateien werden gelöscht, neue geöffnet.

Beispiel:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

Im Beispiel oben wird jedes Mal eine neue Datei geöffnet, wenn eine Datei 4 MiB erreicht, und insgesamt werden 10 Dateien aufbewahrt. So läuft der Speicher nie endlos voll.

---

32- Adb Logcat -P Filterskript

Gibt ein eigenes Filterskript für logcat an. Das ist Androids eigenes Filterformat und wird meist von Systementwicklern genutzt. Normale App-Entwickler brauchen es selten.

Beispiel:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Zeitstempel

Eine Option, die in manchen Formaten jeder Protokollzeile einen Zeitstempel voranstellt. Das Verhalten ändert sich zwischen Android-Versionen. Meist erfüllt sie denselben Zweck wie -v time, nur in anderer Form.

Beispiel:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Zeigt jede Zeile im printf-Format, also Priorität (PID) Tag: Nachricht. Dient der Kompatibilität mit älteren Werkzeugen, wird auf modernem Android aber selten gebraucht.

Beispiel:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Zeigt nur die rohe Nachricht ohne Metadaten. Es wird nur das Protokoll selbst geschrieben, ohne Tag, Priorität oder Zeitstempel. Nützlich, wenn die Ausgabe an ein anderes Programm geht.

Beispiel:

```
adb logcat -v raw
```

Beispielausgabe:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Schreibt am Zeilenanfang nur das Tag, danach den Rest. Priorität und PID werden ausgeblendet. Praktisch, wenn man sich auf ein Tag konzentriert.

Beispiel:

```
adb logcat -v tag
```

Beispielausgabe:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Schreibt am Zeilenanfang zuerst die PID und dann das Tag. Praktisch für die Analyse pro Prozess; man sieht schnell, welcher Prozess was geschrieben hat.

Beispiel:

```
adb logcat -v process
```

Beispielausgabe:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Schreibt am Zeilenanfang nur die Thread-ID (TID), danach den Rest. In Analysen mit mehreren Threads sieht man so, welcher Thread aktiv ist. Ein kurzes Format.

Beispiel:

```
adb logcat -v thread
```

Beispielausgabe:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D Und -T Zusammen

Holt die Protokolle eines Zeitraums, etwa die der letzten Stunde, und speichert sie in einer Datei.

Beispiel:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Beispiel (in eine Datei):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C Und -D Nacheinander

Ein Ablauf, der erst leert und dann einsammelt. Zum Beispiel vor einem Test leeren und nach dem Test die Protokolle holen.

Beispiel:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat In Eine Datei Und Auf Den Computer

Praktisch, um in einem mobilen Terminal lange aufzuzeichnen. Zuerst wird auf dem Gerät in eine Datei geschrieben, dann wird sie mit adb pull auf den Computer geholt.

Beispiel (auf dem Gerät speichern):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Beispiel (auf den Computer holen):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Mit Grep

In einem mobilen Terminal (Termux) für schnelle Textsuche mit grep. Kann flexibler sein als der Regex-Filter, weil alle Optionen von grep verfügbar sind.

Beispiel:

```
adb logcat | grep "Hata"
```

Beispiel (Groß- und Kleinschreibung egal):

```
adb logcat | grep -I "error"
```

Beispiel (mit Zeilennummern):

```
adb logcat | grep -N "Exception"
```

Beispiel (mehrere Wörter):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Mit Tail

Kürzt den Live-Strom auf seine letzten Zeilen, wenn man statt des ganzen Protokolls nur das Ende sehen will.

Beispiel:

```
adb logcat | tail -100
```

Beispiel (fortlaufend verfolgen):

```
adb logcat | tail -F
```

---

44- Adb Logcat Mit Head

Zeigt die ersten Zeilen. Gut für einen schnellen Blick in eine lange Aufzeichnung.

Beispiel:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Mit Wc

Zeigt die Gesamtzahl der Zeilen, etwa um zu messen, wie viele Protokolle während eines Tests entstanden sind.

Beispiel:

```
adb logcat -d | wc -l
```

Beispiel (nur die Fehler zählen):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Mit Sort

Sortiert Protokolle alphabetisch oder numerisch. Wird meist bei der Analyse verwendet.

Beispiel:

```
adb logcat -d | sort
```

Beispiel (umgekehrte Reihenfolge):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Mit Uniq

Entfernt wiederholte Zeilen. Schreibt eine App ständig denselben Fehler, vereinfacht dieser Befehl die Ausgabe.

Beispiel:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

Das Beispiel oben zählt und zeigt die 20 häufigsten Zeilen. Sehr hilfreich bei der Fehlersuche.

---

48- Adb Logcat Mit Awk

Holt bestimmte Spalten heraus oder bearbeitet den Text, etwa die PID-Spalte im threadtime-Format.

Beispiel (nur die PID-Spalte):

```
adb logcat -v threadtime | awk '{print $3}'
```

Beispiel (nur Tag und Nachricht):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Mit Xargs

Führt für jede Zeile einen Befehl aus, etwa eine Benachrichtigung für jede Fehlerzeile. Wird meist in der Automatisierung genutzt.

Beispiel:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Mit Sed

Ersetzt Text. Ideal, um bestimmte Muster in der Ausgabe zu ändern.

Beispiel:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

Das Beispiel oben schreibt Uygulama an die Stelle jedes MyApp.

---

51- Adb Logcat Mit Tee

Zeigt die Ausgabe auf dem Bildschirm und schreibt sie gleichzeitig in eine Datei. Live-Beobachtung und Aufzeichnung laufen zusammen.

Beispiel:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Mit Watch

Holt Protokolle in festen Abständen, etwa alle 5 Sekunden eine Zusammenfassung.

Beispiel:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Mit Timeout

Beendet logcat nach einer festgelegten Zeit. Sehr nützlich in automatischen Tests.

Beispiel:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Mit Alias

Kürzt häufig verwendete Befehle ab. In Termux wird es in die Datei .bashrc oder .zshrc eingetragen.

Beispiel (einen Alias festlegen):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Danach ruft man ihn nur noch so auf:

```
lce
```

---

55- Adb Logcat Mit Mehreren Geräten

Zielt auf ein bestimmtes Gerät, wenn mehrere verbunden sind. Zuerst werden die Geräte mit adb devices aufgelistet, dann wird eines mit -s gewählt.

Beispiel:

```
adb devices
```

Beispielausgabe:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Beispiel (ein bestimmtes Gerät):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Mit Kabellosem Debugging

Für kabelloses Debugging. Ist das Gerät im selben WLAN, verbindet man sich ohne Kabel. Ab Android 11 unterstützt.

Beispiel (kabelloses Debugging auf dem Gerät einschalten, dann):

```
adb pair 192.168.1.42:37123
```

Beispiel (verbinden):

```
adb connect 192.168.1.42:5555
```

Beispiel (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Mit Bugreport

Erstellt einen ausführlichen Bericht über das gesamte System. Neben logcat laufen viele Werkzeuge wie dumpsys, dmesg und top, und alles landet in einer Zip-Datei. Der gründlichste Weg der Fehlersuche.

Beispiel:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Beispiel (auf den Computer holen):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Mit Dumpsys

Zeigt neben logcat den aktuellen Zustand der Systemdienste, etwa was ActivityManager gerade tut.

Beispiel:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Mit ANR-Traces

Holt die Stacktraces, wenn eine App nicht reagiert (ANR). Die ANR-Einträge liegen in der Datei /Data/Anr/Traces.Txt.

Beispiel:

```
adb shell cat /Data/Anr/Traces.Txt
```

Beispiel (Root nötig):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Mit Tombstones

Für native Abstürze. Im Ordner /Data/Tombstones/ entsteht für jeden Absturz eine Datei. Hier schaut man nach, wenn C++- oder Rust-Code fehlerhaft ist.

Beispiel:

```
adb shell ls /Data/Tombstones/
```

Beispiel (der neueste Tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Mit Systrace

Für systemweite Leistungsverfolgung. Es sammelt nicht nur Protokolle, sondern Tracing-Daten auf Kernel-Ebene. Wird zusammen mit Perfetto verwendet.

Beispiel:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Mit Layout Inspector

Analysiert das Layout einer App auf dem Bildschirm. Wird mit Layout Inspector genutzt, die zugehörigen Protokolle lassen sich aber auch mit logcat verfolgen.

Beispiel:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Mit Memory Profiler

Beobachtet den Speicherverbrauch. Memory Profiler steckt meist in Android Studio, speicherbezogene Protokolle sieht man aber auch in logcat.

Beispiel:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Mit CPU Profiler

Beobachtet die CPU-Auslastung. Für eine einfache Messung eignet sich auch der Befehl top.

Beispiel:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Mit Network Profiler

Beobachtet den Netzwerkverkehr. Bibliotheken wie OkHttp und Retrofit schreiben in logcat, für eine vollständige Analyse braucht man aber Network Profiler.

Beispiel:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Mit GPU Profiler

Beobachtet die GPU-Auslastung. Für eine einfache Messung eignet sich auch der Befehl gfxinfo.

Beispiel:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Mit Battery Historian

Beobachtet den Akkuverbrauch. Man kann auch mit dumpsys battery nachsehen.

Beispiel:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Für WLAN-Diagnose

Zum Debuggen von WLAN-Problemen wie Verbindungsabbrüchen oder schwachem Signal.

Beispiel:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Für Bluetooth-Diagnose

Zum Debuggen von Bluetooth-Problemen wie Kopplungsfehlern oder Verbindungsabbrüchen.

Beispiel:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Für Sensor-Diagnose

Zum Debuggen von Sensorproblemen, etwa bei Beschleunigungssensor, Gyroskop und Magnetometer.

Beispiel:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Für Kamera-Diagnose

Zum Debuggen von Kameraproblemen, etwa beim Fotografieren oder bei Videoaufnahmen.

Beispiel:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Für Audio-Diagnose

Zum Debuggen von Audioproblemen, etwa wenn kein Ton kommt oder das Mikrofon nicht funktioniert.

Beispiel:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Für Standort-Diagnose

Zum Debuggen von Standortproblemen, etwa ohne GPS-Signal oder wenn sich der Standort nicht aktualisiert.

Beispiel:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Für Benachrichtigungs-Diagnose

Zum Debuggen von Benachrichtigungsproblemen, etwa wenn Benachrichtigungen nicht erscheinen oder zur falschen Zeit kommen.

Beispiel:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Für Berechtigungs-Diagnose

Zum Debuggen von Berechtigungsproblemen, etwa wenn eine Berechtigung nicht erteilt oder verweigert wird.

Beispiel:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Für WebView-Diagnose

Zum Debuggen von WebView-Problemen, etwa wenn eine Seite nicht lädt oder ein JavaScript-Fehler auftritt.

Beispiel:

```
adb logcat -s Chromium WebView
```

Beispiel (ausführlicher):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Für OkHttp-Diagnose

Zum Debuggen von OkHttp-Problemen: Netzwerkanfragen, Antworten und Fehler.

Beispiel:

```
adb logcat -s Okhttp
```

Beispiel (alle Stufen):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Für Retrofit-Diagnose

Zum Debuggen von Retrofit-Problemen. Da es meist auf OkHttp aufsetzt, schaut man in dessen Protokolle.

Beispiel:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Für Glide-Diagnose

Zum Debuggen von Problemen mit Glide (Bildladebibliothek), etwa wenn Bilder nicht laden oder Speicherprobleme auftreten.

Beispiel:

```
adb logcat -s Glide
```

---

80- Adb Logcat Für Picasso-Diagnose

Zum Debuggen von Problemen mit Picasso (Bildladebibliothek).

Beispiel:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Für Room-Diagnose

Zum Debuggen von Problemen mit Room (Datenbankbibliothek): Abfragefehler und Schemaprobleme.

Beispiel:

```
adb logcat -s Room
```

---

82- Adb Logcat Für Firebase-Diagnose

Zum Debuggen von Firebase-Problemen, für Module wie Crashlytics, Analytics und Messaging.

Beispiel:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Für Crashlytics-Diagnose

Zum Debuggen von Problemen mit Firebase Crashlytics.

Beispiel:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Für Analytics-Diagnose

Zum Debuggen von Analyseproblemen: nicht gesendete Ereignisse und Probleme beim gebündelten Senden.

Beispiel:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Mit Kabellosem ADB-Debugging

Für kabelloses Debugging muss adb manchmal neu gestartet werden. Zuerst per USB verbinden, dann auf kabellos umschalten.

Beispiel:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Beim Neustart Des Geräts

Startet das Gerät neu und beobachtet dabei die Protokolle, besonders für Fehler beim Hochfahren.

Beispiel (zwei Terminals):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Beim Neustart Einer App

Startet nur eine App neu und beobachtet dabei die Protokolle.

Beispiel:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Beim Installieren Einer App

Installiert eine APK und beobachtet dabei die Protokolle.

Beispiel:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Beim Deinstallieren Einer App

Deinstalliert eine App und beobachtet dabei die Protokolle.

Beispiel:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Mit Bildschirmfoto

Macht im Moment eines Fehlers ein Bildschirmfoto und sichert das Protokoll.

Beispiel:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Mit Bildschirmaufnahme

Nimmt im Moment eines Fehlers den Bildschirm auf und sichert das Protokoll. Ab Android 4.4 unterstützt.

Beispiel:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Mit Monkey-Test

Testet eine App, indem zufällige Benutzereingaben simuliert werden. Zusammen mit den Protokollen sehr wirkungsvoll.

Beispiel:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Mit UI-Automator-Test

Für UI-Automatisierungstests, um die Protokolle während des Tests zu beobachten.

Beispiel:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Mit Espresso-Test

Für Espresso-Tests, eine Teilmenge der UI-Automatisierung.

Beispiel:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Mit JUnit-Test

Für Unit-Tests. Sie laufen meist auf einem Computer, können aber auch auf dem Gerät laufen.

Beispiel:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Mit StrictMode

Erkennt Leistungsprobleme einer App. Warnt, wenn auf dem Hauptthread auf Speicher oder Netzwerk zugegriffen wird.

Beispiel:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Mit Choreographer

Erkennt übersprungene Frames. Hier schaut man nach, wenn Animationen ruckeln.

Beispiel:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Mit GPU-Overdraw

Erkennt Overdraw-Probleme. Wird bei der Analyse der grafischen Leistung verwendet.

Beispiel:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Mit Dem Render-Thread

Erkennt Probleme des Render-Threads. Auf modernem Android wird auf diesem Thread gezeichnet.

Beispiel:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Mit Dem GPU-Renderer

Erkennt Probleme beim GPU-Rendering. Hier schaut man nach, sobald das GPU-Rendering-Profil eingeschaltet ist.

Beispiel:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Mit TextView

Erkennt Probleme mit Textansichten, etwa Schriftprobleme und Messfehler.

Beispiel:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Mit RecyclerView

Erkennt Probleme mit Listenansichten: Scrollleistung und Wiederverwendung von Ansichten.

Beispiel:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Mit ViewPager

Erkennt Probleme mit Seitenübergängen.

Beispiel:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Mit Fragments

Erkennt Probleme im Lebenszyklus von Fragments.

Beispiel:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Mit Activities

Erkennt Probleme im Lebenszyklus von Activities.

Beispiel:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Mit Diensten

Erkennt Probleme mit Hintergrunddiensten.

Beispiel:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Mit WorkManager

Erkennt Probleme mit WorkManager, für Hintergrundaufgaben.

Beispiel:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Mit AlarmManager

Erkennt Probleme mit AlarmManager, für geplante Aufgaben.

Beispiel:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Mit SharedPreferences

Erkennt Probleme mit SharedPreferences (einfache Datenspeicherung).

Beispiel:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Mit SQLite

Erkennt Probleme mit SQLite (der lokalen Datenbank).

Beispiel:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Mit Dateioperationen

Erkennt Probleme beim Lesen und Schreiben von Dateien.

Beispiel:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Mit Netzwerkoperationen

Erkennt Probleme mit Netzwerkverbindungen wie HTTP, TCP und UDP.

Beispiel:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Mit SSL

Erkennt Probleme mit SSL-Zertifikaten. Hier schaut man nach, wenn sichere Verbindungen scheitern.

Beispiel:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Mit JSON

Erkennt Probleme beim Parsen von JSON.

Beispiel:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Mit XML

Erkennt Probleme beim Parsen von XML.

Beispiel:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Mit Bildern

Erkennt Probleme beim Laden und Verarbeiten von Bildern.

Beispiel:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Mit Medien

Erkennt Probleme bei Medienwiedergabe und -aufnahme.

Beispiel:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Mit ExoPlayer

Erkennt Probleme mit ExoPlayer (Mediaplayer-Bibliothek).

Beispiel:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Mit Chromecast

Erkennt Probleme mit Chromecast.

Beispiel:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Mit Werbung

Erkennt Probleme mit Werbebibliotheken.

Beispiel:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Mit Abrechnung

Erkennt Probleme mit der Google-Play-Abrechnung.

Beispiel:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Mit Karten

Erkennt Probleme mit Kartenbibliotheken.

Beispiel:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Mit Anmeldung

Erkennt Anmeldeprobleme, etwa mit Google Sign-In und Firebase Auth.

Beispiel:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Mit Benachrichtigungen

Erkennt Probleme beim Senden und Empfangen von Benachrichtigungen.

Beispiel:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Mit Deep Links

Erkennt Probleme mit Deep Links, etwa beim Öffnen einer App über eine URL.

Beispiel:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Mit Intents

Erkennt Probleme mit Intents beim Wechsel zwischen Apps.

Beispiel:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Mit Broadcasts

Erkennt Probleme mit Broadcast-Nachrichten zwischen System und Apps.

Beispiel:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Mit Content Providern

Erkennt Probleme mit Content Providern, für den Datenaustausch zwischen Apps.

Beispiel:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Mit JobScheduler

Erkennt Probleme mit JobScheduler, für geplante Hintergrundaufgaben.

Beispiel:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Mit Doze-Modus

Erkennt Probleme mit dem Doze-Modus (Ruhemodus), für Hintergrundarbeit, während das Gerät schläft.

Beispiel:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Mit Akkuoptimierung

Erkennt Probleme mit der Akkuoptimierung, für Einschränkungen im Hintergrund.

Beispiel:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Mit Speicherlecks

Erkennt Speicherlecks. Wird zusammen mit Werkzeugen wie LeakCanary verwendet.

Beispiel:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Mit ANRs

Erkennt ANR-Probleme (App reagiert nicht).

Beispiel:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Mit Abstürzen

Erkennt Abstürze.

Beispiel:

```
adb logcat -b crash
```

---

135- Adb Logcat Mit Nativen Abstürzen

Erkennt native Abstürze (C/C++/Rust).

Beispiel:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Mit Tombstone-Analyse

Analysiert die Einträge nativer Abstürze.

Beispiel:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Mit Systrace-Analyse

Analysiert System-Trace-Aufzeichnungen.

Beispiel:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Mit Perfetto-Analyse

Das moderne Werkzeug für System-Tracing, gründlicher als logcat.

Beispiel:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Mit Atrace-Analyse

Für Tracing auf Systemebene; die ältere Form von Perfetto.

Beispiel:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Mit Traceview-Analyse

Für Profiling auf Methodenebene.

Beispiel:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Mit Methoden-Tracing

Verfolgt eine bestimmte Methode.

Beispiel:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Mit Heap-Dump

Erstellt einen Speicherabzug. Sehr nützlich bei der Speicheranalyse.

Beispiel:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Mit GC-Protokoll

Beobachtet Ereignisse der Speicherbereinigung (Garbage Collection).

Beispiel:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Mit JNI-Debugging

Erkennt Probleme mit JNI (Java Native Interface).

Beispiel:

```
adb logcat -s JNI
```

---

145- Adb Logcat Mit ART-Debugging

Erkennt Probleme mit ART (Android Runtime).

Beispiel:

```
adb logcat -s Art
```

---

146- Adb Logcat Mit Dalvik-Debugging

Erkennt Probleme der virtuellen Maschine Dalvik. Wird auf älteren Android-Versionen verwendet.

Beispiel:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat Und Das Chatty-Problem

Schreibt logcat zu viele Protokolle, gibt es eine „chatty“-Warnung aus und überspringt Zeilen. Abhilfe schafft ein Filter oder ein größerer Puffer.

Beispiel (Puffer vergrößern):

```
adb logcat -G 16M
```

Beispiel (Filter verwenden):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Mit Farbiger Ausgabe

In langen Protokollströmen schont farbige Ausgabe die Augen und macht den Fehler sofort sichtbar.

Beispiel:

```
adb logcat -v color
```

Beispiel (mit time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Mit Einem Bash-Skript

Schreibt ein Skript, das Protokolle automatisch sammelt. Darauf lässt sich ein System für Fehlerberichte aufbauen.

Beispielskript:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat Zusammenfassung

Wie diese Liste zeigt, ist logcat kein einzelner Befehl, sondern eine Befehlssammlung. Zusammen mit Filtern, Formaten, Puffern, Dateien und anderen Werkzeugen wird es zur stärksten Waffe der Fehlersuche. Wer weiß, welche Kombination zu welcher Lage passt, findet einen Fehler in Minuten. Merke: Die erste Regel einer Debugging-Sitzung ist immer dieselbe; zuerst mit adb logcat -c leeren, dann den Fehler nachstellen, dann mit gefiltertem logcat beobachten.

@@ Android Debug Bridge Verbindung
@: ADB-Befehle Für Die Kabellose Verbindung
1- Kabelloses Debugging Einschalten

Zuerst müssen auf dem Gerät die Entwickleroptionen eingeschaltet werden. Dazu 7-mal auf Einstellungen → Über das Telefon → Build-Nummer tippen. Dann Einstellungen → System → Entwickleroptionen → Kabelloses Debugging einschalten. Ohne diesen Schritt funktionieren weder adb pair noch adb connect.

Beispielpfad:

```
Einstellungen → Über das Telefon → Build-Nummer (7-mal tippen)
Einstellungen → System → Entwickleroptionen → Kabelloses Debugging (an)
```

---

2- IP-Adresse Und Port Des Geräts Herausfinden

Ist kabelloses Debugging eingeschaltet, öffnet man „Gerät mit einem Kopplungscode koppeln“. Dort erscheinen die IP-Adresse des Geräts und eine zufällige Portnummer, zum Beispiel 192.168.1.42:37123. Dieser Port dient nur dem Koppeln und ändert sich danach. Auf demselben Bildschirm steht auch ein 6-stelliger Kopplungscode.

Beispielansicht:

```
IP-Adresse und Port
192.168.1.42:37123

WLAN-Kopplungscode
482915
```

---

3- Der Befehl Adb Pair

Dieser Befehl koppelt das Gerät. Er wird auf einem Computer oder in Termux ausgeführt. Man gibt die auf dem Gerät angezeigte IP-Adresse und den Kopplungsport ein. Beim Ausführen fragt das Terminal Enter pairing code: und wartet auf den 6-stelligen Code. Ist der Code richtig, erscheint die Meldung Successfully paired.

Beispiel:

```
adb pair 192.168.1.42:37123
```

Beispielausgabe:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Bei einem falschen Code schlägt es so fehl:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Andere Wege, Adb Pair Den Code Zu Geben

Der Code muss nicht jedes Mal von Hand eingegeben werden. Manche Versionen nehmen ihn direkt als Parameter an, aus Sicherheitsgründen wird das aber meist nicht unterstützt. Einen Versuch ist es trotzdem wert.

Beispiel (in einer Zeile):

```
adb pair 192.168.1.42:37123 482915
```

Beispiel (mit einer Pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Diese Wege unterscheiden sich je nach Version. Am zuverlässigsten ist es, den Befehl auszuführen und den Code von Hand einzugeben.

---

5- Worauf Man Bei Adb Pair Achten Muss

Kopplungsport und Verbindungsport sind verschieden. Der Kopplungsport ist meist zufällig und hoch, der Verbindungsport meist 5555. Gekoppelt wird nur einmal; für eine erneute Verbindung mit demselben Gerät reicht adb connect. Setzt man das Gerät jedoch auf Werkseinstellungen zurück oder schaltet kabelloses Debugging aus, wird die Kopplung zurückgesetzt und adb pair ist erneut nötig.

Beispielvergleich:

```
Koppeln:    adb pair 192.168.1.42:37123   → fragt nach einem Code, einmalig
Verbinden:  adb connect 192.168.1.42:5555  → kein Code, jedes Mal
```

---

6- Der Befehl Adb Connect

Dieser Befehl verbindet mit einem gekoppelten Gerät. Man gibt die IP-Adresse und den Port vom Hauptbildschirm für kabelloses Debugging ein. Der Port ist meist 5555, kann aber abweichen. Klappt die Verbindung, erscheint die Meldung Connected to.

Beispiel:

```
adb connect 192.168.1.42:5555
```

Beispielausgabe:

```
Connected to 192.168.1.42:5555
```

Ist es bereits verbunden, warnt es so:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Ohne Port

Manche Versionen probieren ohne Portangabe standardmäßig 5555. Das Verhalten ist aber nicht auf jedem Gerät gleich, daher ist es am sichersten, den Port ausdrücklich anzugeben.

Beispiel (Standardport):

```
adb connect 192.168.1.42
```

Beispiel (ausdrücklicher Port):

```
adb connect 192.168.1.42:5555
```

---

8- Die Verbindung Prüfen

Nach dem Verbinden zeigt der Befehl adb devices, ob das Gerät wirklich verbunden ist. Die IP-Adresse des Geräts sollte mit device daneben in der Liste stehen. Steht dort offline, wurde die Verbindung aufgebaut, ist aber noch nicht stabil; kurz darauf erneut versuchen.

Beispiel:

```
adb devices
```

Beispielausgabe:

```
List of devices attached
192.168.1.42:5555   device
```

Beispielausgabe (nicht verbunden):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Der Befehl Adb Disconnect

Beendet die Verbindung. Die adb-Verbindung endet, auch wenn das Gerät im selben Netz bleibt. Für eine neue Verbindung einfach wieder adb connect ausführen; Koppeln ist nicht nötig.

Beispiel (ein Gerät):

```
adb disconnect 192.168.1.42:5555
```

Beispielausgabe:

```
Disconnected 192.168.1.42:5555
```

Beispiel (alle Geräte):

```
adb disconnect
```

Beispielausgabe:

```
Disconnected everything
```

---

10- Der Befehl Adb Tcpip

Schaltet ein per USB verbundenes Gerät in den kabellosen Modus. Der Befehl lässt das Gerät auf dem angegebenen Port lauschen, meist 5555. Dieser Weg ist bei Versionen vor Android 11 verbreiteter, weil es dort kein adb pair gibt.

Beispiel:

```
adb tcpip 5555
```

Beispielausgabe:

```
Restarting in TCP mode port: 5555
```

Danach das USB-Kabel abziehen und diesen Befehl ausführen:

```
adb connect 192.168.1.42:5555
```

---

11- Der Befehl Adb Usb

Wechselt vom kabellosen Modus zurück in den USB-Modus, wenn man das Gerät wieder per Kabel verbinden will.

Beispiel:

```
adb usb
```

Beispielausgabe:

```
Restarting in USB mode
```

---

12- Adb Kill-Server Und Start-Server

Der adb-Server hängt manchmal oder lehnt Verbindungen ab. Dann muss er beendet und neu gestartet werden. Das setzt alle Verbindungen zurück und sorgt für einen sauberen Start.

Beispiel:

```
adb kill-server
adb start-server
```

Beispielausgabe:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Danach erneut verbinden:

```
adb connect 192.168.1.42:5555
```

---

13- Im Selben Netzwerk Sein

Für kabelloses adb müssen Gerät und Computer oder Telefon im selben WLAN sein. In verschiedenen Netzen kommt keine Verbindung zustande. Gastnetze, die Trennung von 5 GHz und 2,4 GHz, ein VPN oder eine Firewall machen manchmal Ärger. Die einfache Lösung: beide Geräte mit demselben Router verbinden und VPNs ausschalten.

Beispielprüfung:

```
Gerät: Einstellungen → WLAN → Verbundenes Netz (z. B. Home_Wifi)
Termux: Einstellungen → WLAN → Verbundenes Netz (z. B. Home_Wifi)
```

---

14- Die IP-Adresse Von Hand Herausfinden

Die IP-Adresse des Geräts findet man unter Einstellungen → WLAN → Verbundenes Netz → Netzwerkdetails. Oder man schaut in Termux mit dem Befehl unten nach. Die IP-Adresse kann sich aber mit der Zeit ändern, daher ist der Blick auf den Bildschirm für kabelloses Debugging am zuverlässigsten.

Beispiel (Termux):

```
ifconfig wlan0
```

Beispiel (auf dem Gerät):

```
Einstellungen → WLAN → Verbundenes Netz → IP-Adresse
```

---

15- Aus Termux Auf Demselben Gerät Mit Sich Selbst Verbinden

Läuft Termux auf demselben Gerät, ist adb bereits lokal, man kann sich aber trotzdem mit adb connect über localhost verbinden. Das geschieht meist mit 127.0.0.1:5555. Manche Geräte lehnen das ab; dann nimmt man die eigene LAN-IP des Geräts.

Beispiel (localhost):

```
adb connect 127.0.0.1:5555
```

Beispiel (eigene LAN-IP):

```
adb connect 192.168.1.42:5555
```

---

16- Befehle Mit Mehreren Verbundenen Geräten Ausführen

Sind mehrere Geräte gleichzeitig verbunden, wählt man in jedem Befehl das Gerät mit dem Parameter -s. Andernfalls meldet adb error: more than one device.

Beispiel:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Wenn Die Verbindung Abreißt

Wird das WLAN-Signal schwach, schläft das Gerät ein oder ändert sich das Netz, reißt die Verbindung ab. Das Gerät verschwindet dann aus der Liste von adb devices oder erscheint als offline. Lösung: adb connect erneut ausführen. Die Kopplung gilt meist weiter, adb pair ist nicht nötig.

Beispiel für die Wiederherstellung:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Nach Einem Neustart Des Geräts

Nach einem Neustart ist kabelloses Debugging meist ausgeschaltet. Auf manchen Geräten bleibt es an, aber die Kopplung wird zurückgesetzt. Nach dem Start den Bildschirm für kabelloses Debugging öffnen, den neuen Port und Code nehmen, erneut adb pair und dann adb connect ausführen.

Beispielablauf:

```
Gerät neu gestartet → Kabelloses Debugging einschalten
Neuer Port: 42157, neuer Code: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Häufige Fehler Beim Koppeln

Failed: Wrong password → der Code wurde falsch eingegeben.
Failed to connect: Connection refused → der Port ist falsch oder das Gerät lauscht nicht.
Failed: Pairing rejected → das Gerät hat die Kopplung abgelehnt; den Code erneuern.
Timeout → es gibt ein Netzwerkproblem; prüfen, ob beide im selben Netz sind.

Beispielfehler und Lösung:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Lösung: Der Code auf dem Gerät hat sich geändert; den neuen Code nehmen und erneut versuchen.

---

20- Kabelloses Adb Ausschalten

Kabelloses Debugging nach getaner Arbeit auszuschalten, ist aus Sicherheitsgründen eine gute Idee. Man kann es in den Einstellungen ausschalten oder mit adb in den USB-Modus wechseln.

Beispiel:

```
adb usb
adb disconnect
```

Beispiel (über die Einstellungen):

```
Einstellungen → System → Entwickleroptionen → Kabelloses Debugging (aus)
```

---

21- Unterschiede Je Nach Android-Version

Android 11 und neuer: Koppeln mit adb pair ist Pflicht, und ein zufälliger Port wird verwendet.
Android 10 und älter: mit adb tcpip 5555 in den kabellosen Modus wechseln; es gibt kein Koppeln.

Beispielvergleich:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Hinweise Zur Sicherheit

Solange kabelloses Debugging an ist, könnte jemand mit bösen Absichten im selben Netz versuchen, auf das Gerät zuzugreifen. Daher diese Funktion in WLANs, denen man nicht vertraut, ausgeschaltet lassen. Der Kopplungscode dient der Sicherheit, Vorsicht ist trotzdem angebracht. Nach getaner Arbeit ausschalten.

Beispiel für eine Vorsichtsmaßnahme:

```
Wenn nicht in Gebrauch: Einstellungen → Entwickleroptionen → Kabelloses Debugging (aus)
In einem sicheren Netz: nur zu Hause oder im Büro einschalten
```

---

23- Der Ganze Ablauf An Einem Ort

Das Skript unten zeigt alle Schritte von Anfang bis zur fertigen Verbindung. IP und Port einfach an das eigene Gerät anpassen.

Beispielablauf:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Erwartetes Ergebnis:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Ein Kurzbefehl-Skript Für Eine Dauerhafte Verbindung

Wer sich ständig verbindet, kann ein Skript schreiben und sich jedes Mal mit einem einzigen Befehl verbinden. In Termux lässt sich dazu etwa eine Funktion in die Datei ~/.bashrc eintragen.

Beispielfunktion:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "IP:Port und Code vom Gerät eingeben:"
    read -p "Adresse: " ADRES
    adb pair "$ADRES"
    read -p "Verbindungsport (z. B. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Verwendung:

```
wifi_adb
```

---

25- Schlussbemerkung

Kabelloses adb ermöglicht Debugging ohne USB-Kabel und ist besonders praktisch, wenn man direkt auf dem Telefon in Termux arbeitet. Die beiden Grundbefehle sind adb pair und adb connect; der erste koppelt einmal, der zweite wird bei jeder Verbindung verwendet. Die übrigen (adb devices, adb disconnect, adb tcpip, adb kill-server) unterstützen diese beiden Befehle und helfen bei Problemen. Die Regel vom selben Netzwerk nicht vergessen, bei Nichtgebrauch ausschalten und bei Problemen den Server neu starten. Das ist alles.
"#;

const SPANISH: &str = r#"@@ Comandos De Logcat
1- Adb Logcat

Envía en directo al terminal todos los registros del dispositivo. Desde el momento en que se ejecuta el comando, cada línea de registro que producen el sistema, las aplicaciones, los servicios y el kernel aparece en pantalla. Para detenerlo, pulsa Ctrl + C en el teclado. Este comando es la base de la depuración; pero sin filtro puede mostrar cientos de líneas por segundo y encontrar lo que necesitas se vuelve difícil.

Ejemplo de uso:

```
adb logcat
```

Ejemplo de salida:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Muestra los registros ya acumulados y el comando termina. En lugar de un flujo continuo, toma una instantánea de ese momento. Úsalo al preparar un informe de errores, al guardar el estado tras un cierre inesperado o cuando quieras obtener los registros una sola vez desde un script.

Ejemplo de uso:

```
adb logcat -d
```

Ejemplo de uso (guardar en un archivo):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Este comando se usa mucho en scripts que envían informes automáticamente a un servidor, porque termina por sí solo y nunca bloquea el script.

---

3- Adb Logcat -C

Vacía todos los búferes de registro y borra todo lo acumulado hasta ese momento. Si lo ejecutas antes de una nueva sesión de depuración, solo verás los registros producidos desde entonces y el ruido antiguo no te confundirá. Al intentar reproducir un error, suele ser el primer paso.

Ejemplo de uso:

```
adb logcat -c
adb logcat
```

Ejecutando en orden los dos comandos de arriba se obtiene un flujo de registros totalmente limpio. Solo aparece lo que ocurre después de vaciar.

---

4- Adb Logcat -G

Muestra el tamaño de los búferes de registro actuales y cuánto están ocupados. Enumera, línea a línea, la ocupación de cada búfer, su capacidad total y la cantidad usada. Si los registros pasan muy rápido y se descartan entradas antiguas (a esto se le llama «chatty»), mira aquí primero.

Ejemplo de uso:

```
adb logcat -g
```

Ejemplo de salida:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Tamaño

Cambia el tamaño del búfer de registro. Por ejemplo, puedes convertir un búfer de 256 KiB en uno de 16 MiB. Así se guardan más registros y las entradas antiguas se descartan más tarde. Muy útil en pruebas largas o al depurar una aplicación muy ruidosa. El ajuste se restablece al reiniciar el dispositivo.

Ejemplo de uso:

```
adb logcat -G 16M
```

Ejemplo de uso (para reducirlo):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Archivo

Escribe los registros en el archivo indicado en lugar de mostrarlos en el terminal. El terminal queda limpio y el registro se conserva. La ruta del archivo está en el dispositivo, no en un ordenador. En un terminal móvil (Termux) se suele escribir bajo /Sdcard/.

Ejemplo de uso:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Ejemplo de uso (añadiendo al archivo):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

En el ejemplo de arriba se abre un archivo nuevo cuando el archivo llega a 4 MiB, y se conservan 10 archivos en total.

---

7- Adb Logcat -B Main

Vigila solo el búfer principal de las aplicaciones. Aquí están los registros que escriben las aplicaciones con llamadas como Log.d, Log.i y Log.e. Ya se vigila por defecto, pero se indica expresamente cuando solo quieres este búfer. Es donde más se mira al desarrollar una aplicación.

Ejemplo de uso:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Vigila los registros de los servicios del sistema. Aquí están los registros de servicios como ActivityManager, WindowManager, PackageManager y PowerManager. Sirve para ver eventos como el inicio de aplicaciones, la concesión de permisos y la vinculación de servicios. Suele vigilarse junto con main.

Ejemplo de uso:

```
adb logcat -b system
```

Ejemplo de salida:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Vigila solo los registros de cierres inesperados. Aquí están los bloques FATAL EXCEPTION que se escriben cuando una aplicación se cierra a la fuerza. Al depurar un cierre, mirar primero este búfer es lo más rápido, porque el seguimiento de la pila está justo aquí.

Ejemplo de uso:

```
adb logcat -b crash
```

Ejemplo de salida:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Vigila los registros de radio, telefonía, señal y conexión de datos. Útil al trabajar con la tarjeta SIM, el registro en la red, la conexión 4G/5G, los SMS o las llamadas. Normalmente no hace falta en el desarrollo habitual de aplicaciones.

Ejemplo de uso:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Vigila los registros binarios de los eventos del sistema. Este búfer no contiene texto normal, sino registros cortos codificados que dejan las llamadas a Log.event. Aparecen con etiquetas como am_proc_start y am_activity_launch. Se usa en el análisis de rendimiento.

Ejemplo de uso:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Vigila los registros del kernel. Muestra los mensajes que escribe el kernel de Linux, que suelen parecerse a la salida del comando dmesg. Aquí están los errores de controladores, los problemas de memoria y la información sobre temperatura y gestión de energía. Puede requerir root.

Ejemplo de uso:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Vigila todos los búferes a la vez. Main, system, crash, radio, events y kernel fluyen mezclados. Úsalo cuando quieras verlo todo, aunque la salida se vuelve muy abundante, por eso suele combinarse con un filtro.

Ejemplo de uso:

```
adb logcat -b all
```

Ejemplo de uso (con filtro):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Varios Búferes

Vigila a la vez varios búferes separados por comas, por ejemplo para ver juntos los registros de la aplicación y de los cierres. Eligiendo solo los canales que necesitas, reduces la salida.

Ejemplo de uso:

```
adb logcat -b main,crash
```

Ejemplo de uso (tres canales):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

El formato de salida corto predeterminado. Cada línea empieza solo con la prioridad y la etiqueta. Bueno para un vistazo rápido, pero sin marca de tiempo es difícil ordenar los eventos.

Ejemplo de uso:

```
adb logcat -v brief
```

Ejemplo de salida:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Añade la fecha y la hora. Mucho más útil que el formato corto, porque ves cuándo ocurrió cada evento. Por debajo del segundo se muestran los milisegundos.

Ejemplo de uso:

```
adb logcat -v time
```

Ejemplo de salida:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

El formato de salida más recomendado. Incluye fecha, hora, prioridad, etiqueta, PID (identificador de proceso) y TID (identificador de hilo). En aplicaciones con varios hilos es imprescindible para saber qué hilo hizo qué. Es también el formato que Android Studio usa por defecto.

Ejemplo de uso:

```
adb logcat -v threadtime
```

Ejemplo de salida:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Da una salida detallada con todos los campos de metadatos y una línea en blanco entre entradas. En sesiones largas permite ver cada registro como un bloque, aunque ocupa espacio.

Ejemplo de uso:

```
adb logcat -v long
```

Ejemplo de salida:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colorea los registros según su prioridad. Los errores salen en rojo, las advertencias en naranja, la información en verde y la depuración en azul. En un flujo largo la vista va directa al error. El terminal debe admitir colores.

Ejemplo de uso:

```
adb logcat -v color
```

Ejemplo de uso (junto con time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Añade el año al formato de hora. Se usa en registros largos o archivos que abarcan más de un año. Normalmente no hace falta en el desarrollo habitual.

Ejemplo de uso:

```
adb logcat -v year
```

Ejemplo de salida:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Muestra el identificador de usuario (UID) de la aplicación que produjo el registro. Útil cuando varias aplicaciones usan la misma etiqueta. Es un modificador, así que se combina con los demás formatos.

Ejemplo de uso:

```
adb logcat -v uid
```

Ejemplo de salida:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filtro De Etiqueta Y Prioridad

Muestra los registros de una etiqueta dada a partir de una prioridad dada. Las prioridades son: V (Verbose) la más baja, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) y S (Silent) la más alta. Esta forma de filtrar es la función más potente de logcat.

Ejemplo de uso (solo errores):

```
adb logcat *:E
```

Ejemplo de uso (advertencias y superiores):

```
adb logcat *:W
```

Ejemplo de uso (lista de permitidos):

```
adb logcat ActivityManager:I MyApp:D *:S
```

El último ejemplo muestra ActivityManager desde info y MyApp desde debug, y silencia todas las demás etiquetas. En algunos shells el carácter * es especial, así que hay que ponerlo entre comillas:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Etiqueta

Un atajo del filtro silencioso. Muestra solo la etiqueta indicada y silencia todo lo demás. La forma más práctica de centrarte en tu propia etiqueta mientras desarrollas.

Ejemplo de uso:

```
adb logcat -s MyApp
```

Ejemplo de uso (varias etiquetas):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

En el ejemplo de arriba solo fluyen los registros de tres etiquetas; todo lo demás queda en silencio.

---

24- Adb Logcat -S Con Prioridad

Sirve para indicar juntas la etiqueta y la prioridad, por ejemplo para ver solo las advertencias y superiores de la etiqueta MyApp.

Ejemplo de uso:

```
adb logcat MyApp:W *:S
```

Este comando muestra solo los registros warning, error y fatal con la etiqueta MyApp.

---

25- Adb Logcat Filtro --Pid

Muestra solo los registros de un identificador de proceso (PID) concreto. Para saber el PID de una aplicación, primero se usa adb shell pidof y luego se filtra por ese PID. Perfecto para aislar todos los registros de una sesión.

Ejemplo de uso (obtener el PID):

```
adb shell pidof com.example.app
```

Ejemplo de salida:

```
5678
```

Ejemplo de uso (filtrar):

```
adb logcat --pid=5678
```

Ejemplo de uso (en una línea):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Hora

Muestra los registros a partir de la marca de tiempo indicada, es decir, «tráeme lo que viene después de este momento». Se escribe con T mayúscula; la t minúscula significa otra cosa.

Ejemplo de uso:

```
adb logcat -T "09-21 14:00:00.000"
```

Ejemplo de uso (desde hoy):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Número De Líneas

Con t minúscula muestra el número indicado de líneas más recientes y el comando termina. Se comporta como -d, pero solo da las últimas N líneas. Ideal para un vistazo rápido.

Ejemplo de uso (últimas 100 líneas):

```
adb logcat -t 100
```

Ejemplo de uso (últimas 500 líneas y un filtro):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Busca en el contenido con una expresión regular (regex) y muestra las líneas que coinciden. Muy potente para búsquedas rápidas de texto; sobre todo al buscar varias palabras a la vez es más rápido que filtrar.

Ejemplo de uso (una palabra):

```
adb logcat -e "Login"
```

Ejemplo de uso (varias palabras):

```
adb logcat -e "Login|Error|Crash"
```

Ejemplo de uso (con nombre de paquete):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Salida Binaria

Escribe los registros en formato binario. Las personas no pueden leerlo, pero otro logcat puede volver a leerlo u otras herramientas pueden procesarlo. Se usa en sistemas de análisis automático.

Ejemplo de uso:

```
adb logcat -B > Ham_Log.Bin
```

Más tarde ese archivo se lee así:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Estadísticas

Muestra estadísticas de los registros: cuántos registros produjo cada etiqueta y qué etiquetas dispararon la advertencia «chatty». Si crees que una aplicación está inundando el registro, puedes confirmarlo con este comando.

Ejemplo de uso:

```
adb logcat -S
```

Ejemplo de salida:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R Y -N Juntos

Para registros rotativos. -r fija el tamaño de cada archivo y -n cuántos archivos se conservan. Útil cuando una aplicación va a funcionar mucho tiempo y quieres registrar de forma continua. Los archivos antiguos se borran y se abren otros nuevos.

Ejemplo de uso:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

En el ejemplo de arriba se abre un archivo nuevo cada vez que uno llega a 4 MiB y se conservan 10 archivos en total. Así el disco nunca se llena sin fin.

---

32- Adb Logcat -P Script De Filtro

Indica un script de filtro personalizado para logcat. Es el formato de filtro propio de Android y suelen usarlo los desarrolladores del sistema. Los desarrolladores de aplicaciones lo usan muy poco.

Ejemplo de uso:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Marcas De Tiempo

Una opción que, en algunos formatos, añade una marca de tiempo al principio de cada línea. Su comportamiento cambia entre versiones de Android. Suele hacer lo mismo que -v time, pero con otra forma.

Ejemplo de uso:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Muestra cada línea en formato printf, es decir, prioridad (PID) etiqueta: mensaje. Sirve para la compatibilidad con herramientas antiguas, pero rara vez hace falta en Android moderno.

Ejemplo de uso:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Muestra solo el mensaje en bruto, sin metadatos. Se escribe solo el registro, sin etiqueta, prioridad ni marca de tiempo. Útil para pasar la salida a otro programa.

Ejemplo de uso:

```
adb logcat -v raw
```

Ejemplo de salida:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Escribe solo la etiqueta al principio de cada línea y luego el resto. La prioridad y el PID se ocultan. Práctico cuando te centras en una etiqueta.

Ejemplo de uso:

```
adb logcat -v tag
```

Ejemplo de salida:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Escribe primero el PID y luego la etiqueta al principio de cada línea. Práctico para analizar por proceso; se ve rápido qué proceso escribió qué.

Ejemplo de uso:

```
adb logcat -v process
```

Ejemplo de salida:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Escribe solo el identificador de hilo (TID) al principio de cada línea y luego el resto. En análisis con varios hilos permite ver qué hilo está activo. Un formato corto.

Ejemplo de uso:

```
adb logcat -v thread
```

Ejemplo de salida:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D Y -T Juntos

Obtiene los registros de un intervalo de tiempo, por ejemplo los de la última hora, para guardarlos en un archivo.

Ejemplo de uso:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Ejemplo de uso (en un archivo):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C Y -D Seguidos

Un flujo que primero vacía y luego recoge. Por ejemplo, vaciar antes de empezar una prueba y recoger los registros al terminarla.

Ejemplo de uso:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Guardado En Un Archivo Y Llevado Al Ordenador

Práctico para registrar durante mucho tiempo en un terminal móvil. Primero se escribe en un archivo del dispositivo y luego se lleva al ordenador con adb pull.

Ejemplo de uso (guardar en el dispositivo):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Ejemplo de uso (llevar al ordenador):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Con Grep

Se usa en un terminal móvil (Termux) para buscar texto rápidamente con grep. Puede ser más flexible que el filtro regex, porque tienes todas las opciones de grep.

Ejemplo de uso:

```
adb logcat | grep "Hata"
```

Ejemplo de uso (sin distinguir mayúsculas):

```
adb logcat | grep -I "error"
```

Ejemplo de uso (con números de línea):

```
adb logcat | grep -N "Exception"
```

Ejemplo de uso (varias palabras):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Con Tail

Reduce el flujo en directo a sus últimas líneas, cuando solo quieres ver el final en lugar de todo el registro.

Ejemplo de uso:

```
adb logcat | tail -100
```

Ejemplo de uso (seguimiento continuo):

```
adb logcat | tail -F
```

---

44- Adb Logcat Con Head

Muestra las primeras líneas. Bueno para echar un vistazo rápido a un registro largo.

Ejemplo de uso:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Con Wc

Muestra el número total de líneas, por ejemplo para medir cuántos registros se produjeron durante una prueba.

Ejemplo de uso:

```
adb logcat -d | wc -l
```

Ejemplo de uso (contar solo los errores):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Con Sort

Ordena los registros alfabética o numéricamente. Se usa sobre todo durante el análisis.

Ejemplo de uso:

```
adb logcat -d | sort
```

Ejemplo de uso (orden inverso):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Con Uniq

Elimina las líneas repetidas. Si una aplicación escribe una y otra vez el mismo error, este comando simplifica la salida.

Ejemplo de uso:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

El ejemplo de arriba cuenta y muestra las 20 líneas más repetidas. Muy útil al depurar.

---

48- Adb Logcat Con Awk

Extrae columnas concretas o trabaja sobre el texto, por ejemplo la columna PID en formato threadtime.

Ejemplo de uso (solo la columna PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Ejemplo de uso (solo etiqueta y mensaje):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Con Xargs

Ejecuta un comando por cada línea, por ejemplo enviar una notificación por cada línea de error. Se usa sobre todo en automatización.

Ejemplo de uso:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Con Sed

Sustituye texto. Ideal para cambiar patrones concretos en la salida.

Ejemplo de uso:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

El ejemplo de arriba escribe Uygulama en lugar de cada MyApp.

---

51- Adb Logcat Con Tee

Muestra en pantalla y escribe en un archivo a la vez. El seguimiento en directo y la grabación ocurren juntos.

Ejemplo de uso:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Con Watch

Obtiene registros a intervalos fijos, por ejemplo un resumen cada 5 segundos.

Ejemplo de uso:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Con Timeout

Detiene logcat tras un tiempo fijado. Muy útil en pruebas automáticas.

Ejemplo de uso:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Con Alias

Acorta los comandos que usas a menudo. En Termux se añade al archivo .bashrc o .zshrc.

Ejemplo de uso (definir un alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Después basta con llamarlo así:

```
lce
```

---

55- Adb Logcat Con Varios Dispositivos

Apunta a un dispositivo concreto cuando hay varios conectados. Primero se listan los dispositivos con adb devices y luego se elige uno con -s.

Ejemplo de uso:

```
adb devices
```

Ejemplo de salida:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Ejemplo de uso (un dispositivo concreto):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Con Depuración Inalámbrica

Para la depuración inalámbrica. Cuando el dispositivo está en la misma red Wi-Fi, te conectas sin cable. Compatible desde Android 11.

Ejemplo de uso (activa la depuración inalámbrica en el dispositivo y luego):

```
adb pair 192.168.1.42:37123
```

Ejemplo de uso (conectar):

```
adb connect 192.168.1.42:5555
```

Ejemplo de uso (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Con Bugreport

Genera un informe detallado de todo el sistema. Además de logcat se ejecutan muchas herramientas como dumpsys, dmesg y top, y todo se reúne en un zip. La forma de depuración más completa.

Ejemplo de uso:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Ejemplo de uso (llevar al ordenador):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Con Dumpsys

Muestra, junto a logcat, el estado actual de los servicios del sistema, por ejemplo qué está haciendo ActivityManager en ese momento.

Ejemplo de uso:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Con Trazas ANR

Obtiene las trazas de pila cuando una aplicación no responde (ANR). Los registros ANR están en el archivo /Data/Anr/Traces.Txt.

Ejemplo de uso:

```
adb shell cat /Data/Anr/Traces.Txt
```

Ejemplo de uso (requiere root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Con Tombstones

Para los cierres nativos. En la carpeta /Data/Tombstones/ se crea un archivo por cada cierre. Se mira aquí cuando falla código C++ o Rust.

Ejemplo de uso:

```
adb shell ls /Data/Tombstones/
```

Ejemplo de uso (el tombstone más reciente):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Con Systrace

Para el seguimiento del rendimiento de todo el sistema. Recoge datos de seguimiento a nivel del kernel, no solo registros. Se usa junto con Perfetto.

Ejemplo de uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Con Layout Inspector

Analiza la disposición de una aplicación en pantalla. Se usa con Layout Inspector, pero también puedes seguir los registros relacionados con logcat.

Ejemplo de uso:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Con Memory Profiler

Vigila el uso de memoria. Memory Profiler suele estar en Android Studio, pero los registros de memoria también se ven en logcat.

Ejemplo de uso:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Con CPU Profiler

Vigila el uso de la CPU. Para una medición sencilla también sirve el comando top.

Ejemplo de uso:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Con Network Profiler

Vigila el tráfico de red. Bibliotecas como OkHttp y Retrofit escriben en logcat, pero un análisis completo requiere Network Profiler.

Ejemplo de uso:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Con GPU Profiler

Vigila el uso de la GPU. Para una medición sencilla también sirve el comando gfxinfo.

Ejemplo de uso:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Con Battery Historian

Vigila el consumo de batería. También se puede consultar con dumpsys battery.

Ejemplo de uso:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Para Diagnosticar El Wi-Fi

Sirve para depurar problemas de Wi-Fi, como cortes de conexión o una señal débil.

Ejemplo de uso:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Para Diagnosticar El Bluetooth

Sirve para depurar problemas de Bluetooth, como fallos de emparejamiento o desconexiones.

Ejemplo de uso:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Para Diagnosticar Sensores

Sirve para depurar problemas de sensores como el acelerómetro, el giroscopio y el magnetómetro.

Ejemplo de uso:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Para Diagnosticar La Cámara

Sirve para depurar problemas de cámara, como al hacer fotos o grabar vídeo.

Ejemplo de uso:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Para Diagnosticar El Audio

Sirve para depurar problemas de audio, como que no suene nada o que el micrófono no funcione.

Ejemplo de uso:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Para Diagnosticar La Ubicación

Sirve para depurar problemas de ubicación, como no recibir señal GPS o que la ubicación no se actualice.

Ejemplo de uso:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Para Diagnosticar Las Notificaciones

Sirve para depurar problemas de notificaciones, como que no aparezcan o lleguen a destiempo.

Ejemplo de uso:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Para Diagnosticar Los Permisos

Sirve para depurar problemas de permisos, como que un permiso no se conceda o se rechace.

Ejemplo de uso:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Para Diagnosticar WebView

Sirve para depurar problemas de WebView, como que una página no cargue o haya un error de JavaScript.

Ejemplo de uso:

```
adb logcat -s Chromium WebView
```

Ejemplo de uso (más detalle):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Para Diagnosticar OkHttp

Sirve para depurar problemas de OkHttp: solicitudes de red, respuestas y errores.

Ejemplo de uso:

```
adb logcat -s Okhttp
```

Ejemplo de uso (todos los niveles):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Para Diagnosticar Retrofit

Sirve para depurar problemas de Retrofit. Como suele basarse en OkHttp, se miran los registros de este.

Ejemplo de uso:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Para Diagnosticar Glide

Sirve para depurar problemas de Glide (biblioteca de carga de imágenes), como imágenes que no cargan o problemas de memoria.

Ejemplo de uso:

```
adb logcat -s Glide
```

---

80- Adb Logcat Para Diagnosticar Picasso

Sirve para depurar problemas de Picasso (biblioteca de carga de imágenes).

Ejemplo de uso:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Para Diagnosticar Room

Sirve para depurar problemas de Room (biblioteca de base de datos): errores de consulta y problemas de esquema.

Ejemplo de uso:

```
adb logcat -s Room
```

---

82- Adb Logcat Para Diagnosticar Firebase

Sirve para depurar problemas de Firebase, en módulos como Crashlytics, Analytics y Messaging.

Ejemplo de uso:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Para Diagnosticar Crashlytics

Sirve para depurar problemas de Firebase Crashlytics.

Ejemplo de uso:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Para Diagnosticar Analytics

Sirve para depurar problemas de analítica: eventos que no se envían y problemas de envío por lotes.

Ejemplo de uso:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Con Depuración ADB Inalámbrica

La depuración inalámbrica a veces requiere reiniciar adb. Conéctate primero por USB y luego pasa a inalámbrico.

Ejemplo de uso:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Al Reiniciar El Dispositivo

Reinicia el dispositivo y vigila los registros, sobre todo para errores que ocurren durante el arranque.

Ejemplo de uso (dos terminales):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Al Reiniciar Una Aplicación

Reinicia solo una aplicación y vigila los registros.

Ejemplo de uso:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Al Instalar Una Aplicación

Instala un APK y vigila los registros.

Ejemplo de uso:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Al Desinstalar Una Aplicación

Desinstala una aplicación y vigila los registros.

Ejemplo de uso:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Con Captura De Pantalla

Hace una captura de pantalla y guarda el registro en el momento de un error.

Ejemplo de uso:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Con Grabación De Pantalla

Graba la pantalla y guarda el registro en el momento de un error. Compatible desde Android 4.4.

Ejemplo de uso:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Con Prueba Monkey

Prueba una aplicación simulando interacciones aleatorias del usuario. Muy eficaz junto con los registros.

Ejemplo de uso:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Con Prueba UI Automator

Para pruebas de automatización de la interfaz, vigilando los registros durante la prueba.

Ejemplo de uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Con Prueba Espresso

Para pruebas Espresso, un subconjunto de la automatización de la interfaz.

Ejemplo de uso:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Con Prueba JUnit

Para pruebas unitarias. Suelen ejecutarse en un ordenador, pero también pueden ejecutarse en el dispositivo.

Ejemplo de uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Con StrictMode

Detecta problemas de rendimiento de una aplicación. Avisa cuando se accede al disco o a la red en el hilo principal.

Ejemplo de uso:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Con Choreographer

Detecta fotogramas saltados. Se mira aquí cuando las animaciones van a tirones.

Ejemplo de uso:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Con Sobredibujado De GPU

Detecta problemas de sobredibujado (overdraw). Se usa en el análisis del rendimiento gráfico.

Ejemplo de uso:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Con El Hilo De Renderizado

Detecta problemas del hilo de renderizado. En Android moderno el dibujo se hace en este hilo.

Ejemplo de uso:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Con El Renderizado De GPU

Detecta problemas de renderizado de GPU. Se mira aquí una vez activado el perfil de renderizado de GPU.

Ejemplo de uso:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Con TextView

Detecta problemas de las vistas de texto, como fallos de fuente y errores de medida.

Ejemplo de uso:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Con RecyclerView

Detecta problemas de las vistas de lista: rendimiento del desplazamiento y reciclaje de vistas.

Ejemplo de uso:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Con ViewPager

Detecta problemas en las transiciones entre páginas.

Ejemplo de uso:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Con Fragments

Detecta problemas del ciclo de vida de los fragments.

Ejemplo de uso:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Con Activities

Detecta problemas del ciclo de vida de las activities.

Ejemplo de uso:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Con Servicios

Detecta problemas de los servicios en segundo plano.

Ejemplo de uso:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Con WorkManager

Detecta problemas de WorkManager, para tareas en segundo plano.

Ejemplo de uso:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Con AlarmManager

Detecta problemas de AlarmManager, para tareas programadas.

Ejemplo de uso:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Con SharedPreferences

Detecta problemas de SharedPreferences (almacenamiento sencillo de datos).

Ejemplo de uso:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Con SQLite

Detecta problemas de SQLite (la base de datos local).

Ejemplo de uso:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Con Operaciones De Archivos

Detecta problemas al leer y escribir archivos.

Ejemplo de uso:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Con Operaciones De Red

Detecta problemas de conexión de red, como HTTP, TCP y UDP.

Ejemplo de uso:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Con SSL

Detecta problemas de certificados SSL. Se mira aquí cuando fallan las conexiones seguras.

Ejemplo de uso:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Con JSON

Detecta problemas al analizar JSON.

Ejemplo de uso:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Con XML

Detecta problemas al analizar XML.

Ejemplo de uso:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Con Imágenes

Detecta problemas al cargar y procesar imágenes.

Ejemplo de uso:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Con Multimedia

Detecta problemas de reproducción y grabación multimedia.

Ejemplo de uso:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Con ExoPlayer

Detecta problemas de ExoPlayer (biblioteca de reproductor multimedia).

Ejemplo de uso:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Con Chromecast

Detecta problemas de Chromecast.

Ejemplo de uso:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Con Anuncios

Detecta problemas de las bibliotecas de publicidad.

Ejemplo de uso:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Con Facturación

Detecta problemas de facturación de Google Play.

Ejemplo de uso:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Con Mapas

Detecta problemas de las bibliotecas de mapas.

Ejemplo de uso:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Con Autenticación

Detecta problemas de inicio de sesión, como Google Sign-In y Firebase Auth.

Ejemplo de uso:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Con Notificaciones

Detecta problemas al enviar y recibir notificaciones.

Ejemplo de uso:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Con Enlaces Profundos

Detecta problemas de enlaces profundos (deep links), como abrir una aplicación desde una URL.

Ejemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Con Intents

Detecta problemas de intents al pasar de una aplicación a otra.

Ejemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Con Broadcasts

Detecta problemas de los mensajes broadcast entre el sistema y las aplicaciones.

Ejemplo de uso:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Con Content Providers

Detecta problemas de content providers, para compartir datos entre aplicaciones.

Ejemplo de uso:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Con JobScheduler

Detecta problemas de JobScheduler, para tareas programadas en segundo plano.

Ejemplo de uso:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Con El Modo Doze

Detecta problemas del modo Doze (reposo), para el trabajo en segundo plano mientras el dispositivo duerme.

Ejemplo de uso:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Con La Optimización De Batería

Detecta problemas de optimización de batería, para las restricciones en segundo plano.

Ejemplo de uso:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Con Fugas De Memoria

Detecta fugas de memoria. Se usa junto con herramientas como LeakCanary.

Ejemplo de uso:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Con ANR

Detecta problemas de ANR (la aplicación no responde).

Ejemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Con Cierres Inesperados

Detecta cierres inesperados.

Ejemplo de uso:

```
adb logcat -b crash
```

---

135- Adb Logcat Con Cierres Nativos

Detecta cierres nativos (C/C++/Rust).

Ejemplo de uso:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Con Análisis De Tombstones

Analiza los registros de cierres nativos.

Ejemplo de uso:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Con Análisis De Systrace

Analiza los registros de seguimiento del sistema.

Ejemplo de uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Con Análisis De Perfetto

La herramienta moderna de seguimiento del sistema, más completa que logcat.

Ejemplo de uso:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Con Análisis De Atrace

Para el seguimiento a nivel de sistema; la forma antigua de Perfetto.

Ejemplo de uso:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Con Análisis De Traceview

Para el perfilado a nivel de método.

Ejemplo de uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Con Seguimiento De Métodos

Sigue un método concreto.

Ejemplo de uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Con Volcado Del Heap

Hace un volcado de memoria. Muy útil en el análisis de memoria.

Ejemplo de uso:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Con El Registro Del GC

Vigila los eventos de recolección de basura (garbage collection).

Ejemplo de uso:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Con Depuración De JNI

Detecta problemas de JNI (Java Native Interface).

Ejemplo de uso:

```
adb logcat -s JNI
```

---

145- Adb Logcat Con Depuración De ART

Detecta problemas de ART (Android Runtime).

Ejemplo de uso:

```
adb logcat -s Art
```

---

146- Adb Logcat Con Depuración De Dalvik

Detecta problemas de la máquina virtual Dalvik. Se usa en versiones antiguas de Android.

Ejemplo de uso:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat Y El Problema Chatty

Cuando logcat escribe demasiados registros, da una advertencia «chatty» y se salta líneas. Se soluciona con un filtro o un búfer más grande.

Ejemplo de uso (ampliar el búfer):

```
adb logcat -G 16M
```

Ejemplo de uso (usar un filtro):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Con Salida En Color

En flujos de registro largos, la salida en color cansa menos la vista y deja ver el error al instante.

Ejemplo de uso:

```
adb logcat -v color
```

Ejemplo de uso (con time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Con Un Script De Bash

Sirve para escribir un script que recoja registros automáticamente. Sobre él se puede montar un sistema de informes de errores.

Script de ejemplo:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat En Resumen

Como muestra esta lista, logcat no es un único comando, sino un conjunto de comandos. Combinado con filtros, formatos, búferes, archivos y otras herramientas se convierte en el arma más potente de la depuración. Si sabes qué combinación encaja en cada situación, encontrar un error lleva minutos. Recuerda: la primera regla de una sesión de depuración es siempre la misma; primero vacía con adb logcat -c, después reproduce el error y luego observa con un logcat filtrado.

@@ Conexión De Android Debug Bridge
@: Comandos De Conexión ADB Inalámbrica
1- Activar La Depuración Inalámbrica

Primero hay que activar las opciones para desarrolladores en el dispositivo. Para ello, toca 7 veces Ajustes → Información del teléfono → Número de compilación. Después activa Ajustes → Sistema → Opciones para desarrolladores → Depuración inalámbrica. Sin este paso no funciona ninguno de los comandos adb pair ni adb connect.

Ruta de ejemplo:

```
Ajustes → Información del teléfono → Número de compilación (tocar 7 veces)
Ajustes → Sistema → Opciones para desarrolladores → Depuración inalámbrica (activada)
```

---

2- Conocer La Dirección IP Y El Puerto Del Dispositivo

Con la depuración inalámbrica activada, entra en la opción «Vincular dispositivo con código de sincronización». Ahí aparecen la dirección IP del dispositivo y un número de puerto aleatorio, por ejemplo 192.168.1.42:37123. Este puerto solo sirve para vincular y cambia después. En la misma pantalla también aparece un código de vinculación de 6 dígitos.

Vista de ejemplo:

```
Dirección IP y puerto
192.168.1.42:37123

Código de vinculación Wi-Fi
482915
```

---

3- El Comando Adb Pair

Este comando vincula el dispositivo. Se ejecuta en un ordenador o en Termux. Escribe la dirección IP y el puerto de vinculación que muestra el dispositivo. Al ejecutarlo, el terminal pregunta Enter pairing code: y espera el código de 6 dígitos. Si el código es correcto, aparece el mensaje Successfully paired.

Ejemplo de uso:

```
adb pair 192.168.1.42:37123
```

Ejemplo de salida:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Si se introduce un código incorrecto, falla así:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Otras Formas De Dar El Código A Adb Pair

No es obligatorio escribir el código a mano cada vez. Algunas versiones lo aceptan directamente como parámetro, aunque por seguridad normalmente no se admite. Aun así se puede probar.

Ejemplo de uso (en una línea):

```
adb pair 192.168.1.42:37123 482915
```

Ejemplo de uso (con una tubería):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Estos métodos cambian según la versión. Lo más fiable es ejecutar el comando y escribir el código a mano.

---

5- Qué Tener En Cuenta Con Adb Pair

El puerto de vinculación y el de conexión son distintos. El de vinculación suele ser aleatorio y alto; el de conexión suele ser 5555. La vinculación se hace una sola vez; para volver a conectar con el mismo dispositivo basta con adb connect. Pero si restableces el dispositivo de fábrica o desactivas la depuración inalámbrica, la vinculación se pierde y hace falta adb pair de nuevo.

Comparación de ejemplo:

```
Vincular:  adb pair 192.168.1.42:37123   → pide un código, una sola vez
Conectar:  adb connect 192.168.1.42:5555  → sin código, cada vez
```

---

6- El Comando Adb Connect

Este comando conecta con un dispositivo vinculado. Escribe la dirección IP y el puerto que aparecen en la pantalla principal de depuración inalámbrica. El puerto suele ser 5555, aunque a veces cambia. Si la conexión funciona, aparece el mensaje Connected to.

Ejemplo de uso:

```
adb connect 192.168.1.42:5555
```

Ejemplo de salida:

```
Connected to 192.168.1.42:5555
```

Si ya está conectado, avisa así:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Sin Puerto

En algunas versiones se prueba 5555 por defecto cuando no se indica puerto. Pero no ocurre igual en todos los dispositivos, así que lo más seguro es escribir el puerto expresamente.

Ejemplo de uso (puerto por defecto):

```
adb connect 192.168.1.42
```

Ejemplo de uso (puerto explícito):

```
adb connect 192.168.1.42:5555
```

---

8- Comprobar La Conexión

Tras conectar, el comando adb devices muestra si el dispositivo está realmente conectado. La dirección IP del dispositivo debe aparecer en la lista con device al lado. Si pone offline, la conexión se estableció pero aún no es estable; vuelve a intentarlo en breve.

Ejemplo de uso:

```
adb devices
```

Ejemplo de salida:

```
List of devices attached
192.168.1.42:5555   device
```

Ejemplo de salida (no conectado):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- El Comando Adb Disconnect

Termina la conexión. La conexión adb termina aunque el dispositivo siga en la misma red. Para volver a conectar, ejecuta de nuevo adb connect; no hace falta vincular.

Ejemplo de uso (un dispositivo):

```
adb disconnect 192.168.1.42:5555
```

Ejemplo de salida:

```
Disconnected 192.168.1.42:5555
```

Ejemplo de uso (todos los dispositivos):

```
adb disconnect
```

Ejemplo de salida:

```
Disconnected everything
```

---

10- El Comando Adb Tcpip

Pasa a modo inalámbrico un dispositivo conectado por USB. Este comando hace que el dispositivo escuche en el puerto indicado, normalmente 5555. Este método es más habitual en versiones anteriores a Android 11, porque en ellas no existe adb pair.

Ejemplo de uso:

```
adb tcpip 5555
```

Ejemplo de salida:

```
Restarting in TCP mode port: 5555
```

Después desconecta el cable USB y ejecuta este comando:

```
adb connect 192.168.1.42:5555
```

---

11- El Comando Adb Usb

Vuelve del modo inalámbrico al modo USB, cuando quieres conectar de nuevo el dispositivo con un cable.

Ejemplo de uso:

```
adb usb
```

Ejemplo de salida:

```
Restarting in USB mode
```

---

12- Adb Kill-Server Y Start-Server

El servidor adb a veces se bloquea o rechaza conexiones. Entonces hay que detenerlo y volver a iniciarlo. Esto restablece todas las conexiones y da un inicio limpio.

Ejemplo de uso:

```
adb kill-server
adb start-server
```

Ejemplo de salida:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Después vuelve a conectar:

```
adb connect 192.168.1.42:5555
```

---

13- Estar En La Misma Red

Para adb inalámbrico, el dispositivo y el ordenador o teléfono deben estar en la misma red Wi-Fi. En redes distintas no hay conexión. Una red de invitados, la separación entre 5 GHz y 2,4 GHz, una VPN o un cortafuegos a veces dan problemas. La solución sencilla: conecta ambos dispositivos al mismo router y desactiva las VPN.

Comprobación de ejemplo:

```
Dispositivo: Ajustes → Wi-Fi → Red conectada (p. ej. Home_Wifi)
Termux: Ajustes → Wi-Fi → Red conectada (p. ej. Home_Wifi)
```

---

14- Conocer La Dirección IP A Mano

Para encontrar la dirección IP del dispositivo, sigue Ajustes → Wi-Fi → Red conectada → Detalles de la red. O consúltala en Termux con el comando de abajo. Pero la dirección IP puede cambiar con el tiempo, así que lo más fiable es mirar cada vez la pantalla de depuración inalámbrica.

Ejemplo de uso (Termux):

```
ifconfig wlan0
```

Ejemplo de uso (en el dispositivo):

```
Ajustes → Wi-Fi → Red conectada → Dirección IP
```

---

15- Conectarse A Sí Mismo Desde Termux En El Mismo Dispositivo

Si Termux se ejecuta en el mismo dispositivo, adb ya es local, pero aun así puedes conectarte por localhost con adb connect. Normalmente se hace con 127.0.0.1:5555. Algunos dispositivos lo rechazan; entonces se usa la IP local del propio dispositivo.

Ejemplo de uso (localhost):

```
adb connect 127.0.0.1:5555
```

Ejemplo de uso (su IP local):

```
adb connect 192.168.1.42:5555
```

---

16- Ejecutar Comandos Con Varios Dispositivos Conectados

Si hay varios dispositivos conectados a la vez, elige el dispositivo con el parámetro -s en cada comando. De lo contrario, adb falla con error: more than one device.

Ejemplo de uso:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Cuando Se Corta La Conexión

Si la señal Wi-Fi se debilita, el dispositivo entra en reposo o cambia la red, la conexión se corta. El dispositivo desaparece entonces de la lista de adb devices o aparece como offline. La solución: volver a ejecutar adb connect. La vinculación suele seguir siendo válida y no hace falta adb pair.

Recuperación de ejemplo:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Tras Reiniciar El Dispositivo

Al reiniciar el dispositivo, la depuración inalámbrica suele desactivarse. En algunos dispositivos sigue activa, pero la vinculación se restablece. Cuando el dispositivo arranque, ve a la pantalla de depuración inalámbrica, toma el nuevo puerto y el nuevo código, ejecuta de nuevo adb pair y después adb connect.

Flujo de ejemplo:

```
Dispositivo reiniciado → activar la depuración inalámbrica
Nuevo puerto: 42157, nuevo código: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Errores Frecuentes Al Vincular

Failed: Wrong password → el código se escribió mal.
Failed to connect: Connection refused → el puerto es incorrecto o el dispositivo no escucha.
Failed: Pairing rejected → el dispositivo rechazó la vinculación; renueva el código.
Timeout → hay un problema de red; comprueba que estáis en la misma red.

Error de ejemplo y solución:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Solución: el código del dispositivo ha cambiado; toma el nuevo código y vuelve a intentarlo.

---

20- Desactivar Adb Inalámbrico

Desactivar la depuración inalámbrica al terminar es una buena idea por seguridad. Puedes desactivarla en los ajustes o pasar al modo USB con adb.

Ejemplo de uso:

```
adb usb
adb disconnect
```

Ejemplo de uso (desde los ajustes):

```
Ajustes → Sistema → Opciones para desarrolladores → Depuración inalámbrica (desactivada)
```

---

21- Diferencias Según La Versión De Android

Android 11 y posteriores: vincular con adb pair es obligatorio y se usa un puerto aleatorio.
Android 10 y anteriores: se pasa a modo inalámbrico con adb tcpip 5555; no hay vinculación.

Comparación de ejemplo:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Notas De Seguridad

Mientras la depuración inalámbrica está activa, alguien malintencionado en la misma red podría intentar acceder al dispositivo. Por eso, mantén esta función desactivada en redes Wi-Fi en las que no confíes. El código de vinculación existe por seguridad, pero conviene ser prudente. Desactívala al terminar.

Precaución de ejemplo:

```
Cuando no la uses: Ajustes → Opciones para desarrolladores → Depuración inalámbrica (desactivada)
En una red segura: actívala solo en casa o en la oficina
```

---

23- Todo El Flujo En Un Solo Lugar

El script de abajo muestra todos los pasos desde cero hasta quedar conectado. Basta con cambiar la IP y el puerto por los de tu dispositivo.

Flujo de ejemplo:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Resultado esperado:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Un Script De Atajo Para Una Conexión Duradera

Si te conectas a menudo, puedes escribir un script y conectarte cada vez con un solo comando. Por ejemplo, en Termux puedes añadir una función al archivo ~/.bashrc.

Función de ejemplo:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Introduce la IP:puerto y el código que muestra el dispositivo:"
    read -p "Dirección: " ADRES
    adb pair "$ADRES"
    read -p "Puerto de conexión (p. ej. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Uso:

```
wifi_adb
```

---

25- Nota Final

Adb inalámbrico permite depurar sin cable USB y resulta especialmente práctico al trabajar en Termux en el propio teléfono. Los dos comandos básicos son adb pair y adb connect; el primero vincula una vez y el segundo se usa en cada conexión. El resto (adb devices, adb disconnect, adb tcpip, adb kill-server) sirve para apoyar esos dos comandos y resolver problemas. No olvides la regla de la misma red, desactívala cuando no la uses y reinicia el servidor si surgen problemas. Eso es todo.
"#;

const FRENCH: &str = r#"@@ Commandes Logcat
1- Adb Logcat

Envoie en direct tous les journaux de l’appareil dans le terminal. Dès que la commande s’exécute, chaque ligne de journal produite par le système, les applications, les services et le noyau s’affiche à l’écran. Appuyez sur Ctrl + C au clavier pour l’arrêter. Cette commande est la base du débogage ; mais sans filtre, elle peut faire défiler des centaines de lignes par seconde, et trouver ce qui vous intéresse devient difficile.

Exemple d’utilisation :

```
adb logcat
```

Exemple de sortie :

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Affiche les journaux déjà accumulés, puis la commande se termine. Au lieu d’un flux continu, elle prend un instantané. Utile pour rédiger un rapport de bug, enregistrer l’état après un plantage ou récupérer les journaux une seule fois depuis un script.

Exemple d’utilisation :

```
adb logcat -d
```

Exemple d’utilisation (enregistrer dans un fichier) :

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Cette commande sert beaucoup dans les scripts qui envoient automatiquement des rapports à un serveur, car elle se termine d’elle-même et ne bloque jamais le script.

---

3- Adb Logcat -C

Vide tous les tampons de journaux et supprime tout ce qui a été accumulé. Lancée avant une nouvelle session de débogage, elle garantit que vous ne voyez que les journaux produits à partir de ce moment, sans bruit ancien pour vous tromper. Pour reproduire un bug, c’est généralement la première étape.

Exemple d’utilisation :

```
adb logcat -c
adb logcat
```

Exécutées dans l’ordre, les deux commandes ci-dessus donnent un flux de journaux parfaitement propre. Seul ce qui se passe après le nettoyage s’affiche.

---

4- Adb Logcat -G

Affiche la taille des tampons de journaux actuels et leur taux de remplissage. Il liste, ligne par ligne, le remplissage de chaque tampon, sa capacité totale et la quantité utilisée. Si les journaux défilent très vite et que les anciennes entrées sont supprimées (on dit « chatty »), commencez par regarder ici.

Exemple d’utilisation :

```
adb logcat -g
```

Exemple de sortie :

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Taille

Modifie la taille du tampon de journaux. Vous pouvez par exemple passer un tampon de 256 Kio à 16 Mio. Plus de journaux sont conservés et les anciennes entrées sont supprimées plus tard. Très utile pendant de longs tests ou pour déboguer une application très bavarde. Le réglage est réinitialisé au redémarrage de l’appareil.

Exemple d’utilisation :

```
adb logcat -G 16M
```

Exemple d’utilisation (pour le réduire) :

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Fichier

Écrit les journaux dans le fichier indiqué au lieu de les afficher dans le terminal. Le terminal reste propre et l’enregistrement est conservé. Le chemin du fichier se trouve sur l’appareil, pas sur un ordinateur. Dans un terminal mobile (Termux), on écrit généralement sous /Sdcard/.

Exemple d’utilisation :

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Exemple d’utilisation (ajout au fichier) :

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

Dans l’exemple ci-dessus, un nouveau fichier est ouvert lorsque le fichier atteint 4 Mio, et 10 fichiers sont conservés au total.

---

7- Adb Logcat -B Main

Surveille uniquement le tampon principal des applications. Les journaux écrits par les applications avec des appels comme Log.d, Log.i et Log.e s’y trouvent. Il est surveillé par défaut, mais on le nomme explicitement quand on ne veut que ce tampon. C’est là qu’on regarde le plus en développant une application.

Exemple d’utilisation :

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Surveille les journaux des services système. Les journaux de services comme ActivityManager, WindowManager, PackageManager et PowerManager s’y trouvent. Il permet de voir des événements comme le lancement d’applications, l’octroi d’autorisations et la liaison de services. On le surveille généralement avec main.

Exemple d’utilisation :

```
adb logcat -b system
```

Exemple de sortie :

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Surveille uniquement les journaux de plantage. Les blocs FATAL EXCEPTION écrits lorsqu’une application est fermée de force s’y trouvent. Pour déboguer un plantage, regarder d’abord ce tampon est le plus rapide, car la trace de pile y figure directement.

Exemple d’utilisation :

```
adb logcat -b crash
```

Exemple de sortie :

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Surveille les journaux liés à la radio, à la téléphonie, au signal et à la connexion de données. Utile pour la carte SIM, l’enregistrement réseau, la connexion 4G/5G, les SMS ou les appels. Rarement nécessaire pour le développement d’applications courant.

Exemple d’utilisation :

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Surveille les journaux binaires des événements système. Ce tampon ne contient pas de texte ordinaire, mais de courts enregistrements codés laissés par les appels Log.event. Ils apparaissent avec des étiquettes comme am_proc_start et am_activity_launch. Utilisé pour l’analyse des performances.

Exemple d’utilisation :

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Surveille les journaux du noyau. Affiche les messages écrits par le noyau Linux, ce qui ressemble généralement à la sortie de la commande dmesg. Les erreurs de pilotes, les problèmes de mémoire, la chaleur et la gestion de l’énergie s’y trouvent. Peut nécessiter root.

Exemple d’utilisation :

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Surveille tous les tampons à la fois. Main, system, crash, radio, events et kernel défilent mélangés. Utile pour tout voir, mais la sortie devient très chargée ; on l’associe donc généralement à un filtre.

Exemple d’utilisation :

```
adb logcat -b all
```

Exemple d’utilisation (filtré) :

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Plusieurs Tampons

Surveille en même temps plusieurs tampons séparés par des virgules, par exemple pour voir ensemble les journaux de l’application et des plantages. En choisissant uniquement les canaux utiles, vous réduisez la sortie.

Exemple d’utilisation :

```
adb logcat -b main,crash
```

Exemple d’utilisation (trois canaux) :

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Le format de sortie court par défaut. Chaque ligne commence seulement par la priorité et l’étiquette. Pratique pour un coup d’œil, mais sans horodatage il est difficile d’ordonner les événements.

Exemple d’utilisation :

```
adb logcat -v brief
```

Exemple de sortie :

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Ajoute la date et l’heure. Bien plus utile que le format court, car on voit quand un événement s’est produit. Les millisecondes sont affichées sous la seconde.

Exemple d’utilisation :

```
adb logcat -v time
```

Exemple de sortie :

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

Le format de sortie le plus recommandé. Il contient la date, l’heure, la priorité, l’étiquette, le PID (identifiant de processus) et le TID (identifiant de thread). Dans les applications multithread, il est indispensable pour savoir quel thread a fait quoi. C’est aussi le format qu’Android Studio utilise par défaut.

Exemple d’utilisation :

```
adb logcat -v threadtime
```

Exemple de sortie :

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Donne une sortie détaillée avec tous les champs de métadonnées et une ligne vide entre les entrées. Dans les longues sessions, chaque journal apparaît comme un bloc, mais cela prend de la place.

Exemple d’utilisation :

```
adb logcat -v long
```

Exemple de sortie :

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colore les journaux selon la priorité. Les erreurs deviennent rouges, les avertissements orange, les infos vertes et le débogage bleu. Dans un long flux, l’œil va tout de suite à l’erreur. Le terminal doit prendre en charge les couleurs.

Exemple d’utilisation :

```
adb logcat -v color
```

Exemple d’utilisation (avec time) :

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Ajoute l’année au format horaire. Pour les enregistrements longs ou les archives couvrant plusieurs années. Rarement nécessaire en développement courant.

Exemple d’utilisation :

```
adb logcat -v year
```

Exemple de sortie :

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Affiche l’identifiant utilisateur (UID) de l’application qui a produit le journal. Utile quand plusieurs applications utilisent la même étiquette. C’est un modificateur, donc il se combine avec les autres formats.

Exemple d’utilisation :

```
adb logcat -v uid
```

Exemple de sortie :

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filtre D’étiquette Et De Priorité

Affiche les journaux d’une étiquette donnée à partir d’une priorité donnée. Les priorités sont : V (Verbose) la plus basse, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) et S (Silent) la plus haute. Ce mode de filtrage est la fonction la plus puissante de logcat.

Exemple d’utilisation (erreurs uniquement) :

```
adb logcat *:E
```

Exemple d’utilisation (avertissements et plus) :

```
adb logcat *:W
```

Exemple d’utilisation (liste blanche) :

```
adb logcat ActivityManager:I MyApp:D *:S
```

Le dernier exemple affiche ActivityManager à partir d’info et MyApp à partir de debug, et fait taire toutes les autres étiquettes. Dans certains shells le caractère * est spécial ; il faut donc le mettre entre guillemets :

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Étiquette

Un raccourci du filtre silencieux. N’affiche que l’étiquette indiquée et fait taire tout le reste. Le moyen le plus pratique de se concentrer sur sa propre étiquette en développant une application.

Exemple d’utilisation :

```
adb logcat -s MyApp
```

Exemple d’utilisation (plusieurs étiquettes) :

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

Dans l’exemple ci-dessus, seuls les journaux de trois étiquettes défilent ; tout le reste reste silencieux.

---

24- Adb Logcat -S Avec Une Priorité

Sert à indiquer ensemble l’étiquette et la priorité, par exemple pour ne voir que les avertissements et plus de l’étiquette MyApp.

Exemple d’utilisation :

```
adb logcat MyApp:W *:S
```

Cette commande n’affiche que les journaux warning, error et fatal portant l’étiquette MyApp.

---

25- Adb Logcat Filtre --Pid

N’affiche que les journaux d’un identifiant de processus (PID) donné. Pour connaître le PID d’une application, on utilise d’abord adb shell pidof, puis on filtre avec ce PID. Parfait pour isoler tous les journaux d’une session.

Exemple d’utilisation (obtenir le PID) :

```
adb shell pidof com.example.app
```

Exemple de sortie :

```
5678
```

Exemple d’utilisation (filtrage) :

```
adb logcat --pid=5678
```

Exemple d’utilisation (sur une ligne) :

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Heure

Affiche les journaux à partir de l’horodatage donné, autrement dit « donne-moi ce qui vient après ce moment ». Il s’écrit avec un T majuscule ; un t minuscule veut dire autre chose.

Exemple d’utilisation :

```
adb logcat -T "09-21 14:00:00.000"
```

Exemple d’utilisation (à partir d’aujourd’hui) :

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Nombre De Lignes

Avec un t minuscule, il affiche le nombre indiqué de lignes les plus récentes, puis la commande se termine. Il se comporte comme -d, mais ne donne que les N dernières lignes. Idéal pour un coup d’œil rapide.

Exemple d’utilisation (100 dernières lignes) :

```
adb logcat -t 100
```

Exemple d’utilisation (500 dernières lignes et un filtre) :

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Recherche dans le contenu avec une expression régulière (regex) et affiche les lignes correspondantes. Très puissant pour une recherche textuelle rapide ; surtout pour plusieurs mots à la fois, c’est plus rapide qu’un filtre.

Exemple d’utilisation (un mot) :

```
adb logcat -e "Login"
```

Exemple d’utilisation (plusieurs mots) :

```
adb logcat -e "Login|Error|Crash"
```

Exemple d’utilisation (avec un nom de paquet) :

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Sortie Binaire

Écrit les journaux au format binaire. Illisible pour un humain, ce format peut être relu par un autre logcat ou traité par d’autres outils. Utilisé dans les systèmes d’analyse automatique.

Exemple d’utilisation :

```
adb logcat -B > Ham_Log.Bin
```

Ce fichier se relit ensuite ainsi :

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Statistiques

Affiche les statistiques des journaux : combien de journaux chaque étiquette a produits et quelles étiquettes ont déclenché l’avertissement « chatty ». Si vous pensez qu’une application inonde le journal, cette commande le confirme.

Exemple d’utilisation :

```
adb logcat -S
```

Exemple de sortie :

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R Et -N Ensemble

Pour la rotation des journaux. -r fixe la taille de chaque fichier et -n le nombre de fichiers conservés. Utile quand une application tourne longtemps et qu’on veut journaliser en continu. Les anciens fichiers sont supprimés et de nouveaux sont ouverts.

Exemple d’utilisation :

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

Dans l’exemple ci-dessus, un nouveau fichier est ouvert chaque fois qu’un fichier atteint 4 Mio, et 10 fichiers sont conservés au total. Ainsi, le disque ne se remplit jamais sans fin.

---

32- Adb Logcat -P Script De Filtre

Indique un script de filtre personnalisé pour logcat. C’est le format de filtre propre à Android, généralement utilisé par les développeurs système. Les développeurs d’applications s’en servent rarement.

Exemple d’utilisation :

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Horodatages

Une option utilisée dans certains formats pour ajouter un horodatage au début de chaque ligne. Son comportement change selon les versions d’Android. Elle fait généralement la même chose que -v time, sous une autre forme.

Exemple d’utilisation :

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Affiche chaque ligne au format printf, soit priorité (PID) étiquette : message. Sert à la compatibilité avec d’anciens outils, rarement utile sur un Android moderne.

Exemple d’utilisation :

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

N’affiche que le message brut, sans aucune métadonnée. Seul le journal lui-même est écrit, sans étiquette, priorité ni horodatage. Utile pour transmettre la sortie à un autre programme.

Exemple d’utilisation :

```
adb logcat -v raw
```

Exemple de sortie :

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Écrit seulement l’étiquette en début de ligne, puis le reste. La priorité et le PID sont masqués. Pratique quand on se concentre sur une étiquette.

Exemple d’utilisation :

```
adb logcat -v tag
```

Exemple de sortie :

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Écrit d’abord le PID puis l’étiquette en début de ligne. Pratique pour une analyse par processus ; on voit vite quel processus a écrit quoi.

Exemple d’utilisation :

```
adb logcat -v process
```

Exemple de sortie :

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Écrit seulement l’identifiant de thread (TID) en début de ligne, puis le reste. Dans une analyse multithread, on voit quel thread est actif. Un format court.

Exemple d’utilisation :

```
adb logcat -v thread
```

Exemple de sortie :

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D Et -T Ensemble

Récupère les journaux d’une plage horaire, par exemple ceux de la dernière heure, pour les enregistrer dans un fichier.

Exemple d’utilisation :

```
adb logcat -d -T "09-21 13:00:00.000"
```

Exemple d’utilisation (dans un fichier) :

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C Puis -D

Un enchaînement qui vide puis récupère. Par exemple, vider avant de lancer un test et récupérer les journaux à la fin du test.

Exemple d’utilisation :

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Enregistré Dans Un Fichier Et Rapatrié Sur Un Ordinateur

Pratique pour enregistrer longtemps dans un terminal mobile. On écrit d’abord dans un fichier sur l’appareil, puis on le récupère sur l’ordinateur avec adb pull.

Exemple d’utilisation (enregistrer sur l’appareil) :

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Exemple d’utilisation (récupérer sur l’ordinateur) :

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Avec Grep

Sert, dans un terminal mobile (Termux), à rechercher rapidement du texte avec grep. Peut être plus souple que le filtre regex, car toutes les options de grep sont disponibles.

Exemple d’utilisation :

```
adb logcat | grep "Hata"
```

Exemple d’utilisation (sans tenir compte de la casse) :

```
adb logcat | grep -I "error"
```

Exemple d’utilisation (avec numéros de ligne) :

```
adb logcat | grep -N "Exception"
```

Exemple d’utilisation (plusieurs mots) :

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Avec Tail

Réduit le flux en direct à ses dernières lignes, quand on ne veut voir que la fin au lieu de tout le journal.

Exemple d’utilisation :

```
adb logcat | tail -100
```

Exemple d’utilisation (suivi continu) :

```
adb logcat | tail -F
```

---

44- Adb Logcat Avec Head

Affiche les premières lignes. Pratique pour jeter un œil à un long enregistrement.

Exemple d’utilisation :

```
adb logcat -d | head -50
```

---

45- Adb Logcat Avec Wc

Affiche le nombre total de lignes, par exemple pour mesurer combien de journaux un test a produits.

Exemple d’utilisation :

```
adb logcat -d | wc -l
```

Exemple d’utilisation (compter seulement les erreurs) :

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Avec Sort

Trie les journaux par ordre alphabétique ou numérique. Généralement utilisé pendant l’analyse.

Exemple d’utilisation :

```
adb logcat -d | sort
```

Exemple d’utilisation (ordre inverse) :

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Avec Uniq

Supprime les lignes répétées. Si une application écrit sans cesse la même erreur, cette commande simplifie la sortie.

Exemple d’utilisation :

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

L’exemple ci-dessus compte et affiche les 20 lignes les plus répétées. Très utile pour le débogage.

---

48- Adb Logcat Avec Awk

Extrait certaines colonnes ou retravaille le texte, par exemple la colonne PID au format threadtime.

Exemple d’utilisation (colonne PID seulement) :

```
adb logcat -v threadtime | awk '{print $3}'
```

Exemple d’utilisation (étiquette et message seulement) :

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Avec Xargs

Exécute une commande pour chaque ligne, par exemple une notification pour chaque ligne d’erreur. Surtout utilisé en automatisation.

Exemple d’utilisation :

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Avec Sed

Remplace du texte. Idéal pour modifier certains motifs dans la sortie.

Exemple d’utilisation :

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

L’exemple ci-dessus écrit Uygulama à la place de chaque MyApp.

---

51- Adb Logcat Avec Tee

Affiche à l’écran et écrit dans un fichier en même temps. Suivi en direct et enregistrement se font ensemble.

Exemple d’utilisation :

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Avec Watch

Récupère les journaux à intervalles réguliers, par exemple un résumé toutes les 5 secondes.

Exemple d’utilisation :

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Avec Timeout

Arrête logcat après une durée donnée. Très utile dans les tests automatiques.

Exemple d’utilisation :

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Avec Alias

Raccourcit les commandes que vous utilisez souvent. Dans Termux, on l’ajoute au fichier .bashrc ou .zshrc.

Exemple d’utilisation (définir un alias) :

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Ensuite, il suffit de l’appeler ainsi :

```
lce
```

---

55- Adb Logcat Avec Plusieurs Appareils

Cible un appareil précis quand plusieurs sont connectés. On liste d’abord les appareils avec adb devices, puis on en choisit un avec -s.

Exemple d’utilisation :

```
adb devices
```

Exemple de sortie :

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Exemple d’utilisation (un appareil précis) :

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Avec Le Débogage Sans Fil

Pour le débogage sans fil. Quand l’appareil est sur le même réseau Wi-Fi, on se connecte sans câble. Pris en charge à partir d’Android 11.

Exemple d’utilisation (activez le débogage sans fil sur l’appareil, puis) :

```
adb pair 192.168.1.42:37123
```

Exemple d’utilisation (connexion) :

```
adb connect 192.168.1.42:5555
```

Exemple d’utilisation (logcat) :

```
adb logcat -v threadtime
```

---

57- Adb Logcat Avec Bugreport

Produit un rapport détaillé de tout le système. En plus de logcat, de nombreux outils comme dumpsys, dmesg et top sont exécutés et rassemblés dans un seul zip. La méthode de débogage la plus complète.

Exemple d’utilisation :

```
adb bugreport /Sdcard/Bugreport.Zip
```

Exemple d’utilisation (récupérer sur l’ordinateur) :

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Avec Dumpsys

Montre, à côté de logcat, l’état actuel des services système, par exemple ce que fait ActivityManager à cet instant.

Exemple d’utilisation :

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Avec Les Traces ANR

Récupère les traces de pile quand une application ne répond pas (ANR). Les enregistrements ANR se trouvent dans le fichier /Data/Anr/Traces.Txt.

Exemple d’utilisation :

```
adb shell cat /Data/Anr/Traces.Txt
```

Exemple d’utilisation (root nécessaire) :

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Avec Les Tombstones

Pour les plantages natifs. Un fichier est créé dans le dossier /Data/Tombstones/ pour chaque plantage. On regarde ici quand du code C++ ou Rust est en défaut.

Exemple d’utilisation :

```
adb shell ls /Data/Tombstones/
```

Exemple d’utilisation (le dernier tombstone) :

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Avec Systrace

Pour le traçage des performances de tout le système. Il collecte des données de traçage au niveau du noyau, pas seulement des journaux. S’utilise avec Perfetto.

Exemple d’utilisation :

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Avec Layout Inspector

Analyse la disposition d’une application à l’écran. S’utilise avec Layout Inspector, mais on peut aussi suivre les journaux associés avec logcat.

Exemple d’utilisation :

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Avec Memory Profiler

Surveille l’utilisation de la mémoire. Memory Profiler se trouve généralement dans Android Studio, mais les journaux liés à la mémoire sont aussi visibles dans logcat.

Exemple d’utilisation :

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Avec CPU Profiler

Surveille l’utilisation du processeur. Pour une mesure simple, la commande top convient aussi.

Exemple d’utilisation :

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Avec Network Profiler

Surveille le trafic réseau. Des bibliothèques comme OkHttp et Retrofit écrivent dans logcat, mais une analyse complète demande Network Profiler.

Exemple d’utilisation :

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Avec GPU Profiler

Surveille l’utilisation du GPU. Pour une mesure simple, la commande gfxinfo convient aussi.

Exemple d’utilisation :

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Avec Battery Historian

Surveille la consommation de la batterie. On peut aussi regarder avec dumpsys battery.

Exemple d’utilisation :

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Pour Diagnostiquer Le Wi-Fi

Sert à déboguer les problèmes de Wi-Fi, comme les coupures de connexion ou un signal faible.

Exemple d’utilisation :

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Pour Diagnostiquer Le Bluetooth

Sert à déboguer les problèmes de Bluetooth, comme les échecs d’appairage ou les déconnexions.

Exemple d’utilisation :

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Pour Diagnostiquer Les Capteurs

Sert à déboguer les problèmes de capteurs, comme l’accéléromètre, le gyroscope et le magnétomètre.

Exemple d’utilisation :

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Pour Diagnostiquer La Caméra

Sert à déboguer les problèmes de caméra, comme la prise de photos ou l’enregistrement vidéo.

Exemple d’utilisation :

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Pour Diagnostiquer L’audio

Sert à déboguer les problèmes audio, comme l’absence de son ou un micro qui ne fonctionne pas.

Exemple d’utilisation :

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Pour Diagnostiquer La Localisation

Sert à déboguer les problèmes de localisation, comme l’absence de signal GPS ou une position qui ne se met pas à jour.

Exemple d’utilisation :

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Pour Diagnostiquer Les Notifications

Sert à déboguer les problèmes de notifications, comme des notifications qui n’apparaissent pas ou arrivent au mauvais moment.

Exemple d’utilisation :

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Pour Diagnostiquer Les Autorisations

Sert à déboguer les problèmes d’autorisations, comme une autorisation non accordée ou refusée.

Exemple d’utilisation :

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Pour Diagnostiquer WebView

Sert à déboguer les problèmes de WebView, comme une page qui ne se charge pas ou une erreur JavaScript.

Exemple d’utilisation :

```
adb logcat -s Chromium WebView
```

Exemple d’utilisation (plus de détails) :

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Pour Diagnostiquer OkHttp

Sert à déboguer les problèmes d’OkHttp : requêtes réseau, réponses et erreurs.

Exemple d’utilisation :

```
adb logcat -s Okhttp
```

Exemple d’utilisation (tous les niveaux) :

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Pour Diagnostiquer Retrofit

Sert à déboguer les problèmes de Retrofit. Comme il repose généralement sur OkHttp, on regarde les journaux de ce dernier.

Exemple d’utilisation :

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Pour Diagnostiquer Glide

Sert à déboguer les problèmes de Glide (bibliothèque de chargement d’images), comme des images qui ne se chargent pas ou des soucis de mémoire.

Exemple d’utilisation :

```
adb logcat -s Glide
```

---

80- Adb Logcat Pour Diagnostiquer Picasso

Sert à déboguer les problèmes de Picasso (bibliothèque de chargement d’images).

Exemple d’utilisation :

```
adb logcat -s Picasso
```

---

81- Adb Logcat Pour Diagnostiquer Room

Sert à déboguer les problèmes de Room (bibliothèque de base de données) : erreurs de requête et problèmes de schéma.

Exemple d’utilisation :

```
adb logcat -s Room
```

---

82- Adb Logcat Pour Diagnostiquer Firebase

Sert à déboguer les problèmes de Firebase, pour des modules comme Crashlytics, Analytics et Messaging.

Exemple d’utilisation :

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Pour Diagnostiquer Crashlytics

Sert à déboguer les problèmes de Firebase Crashlytics.

Exemple d’utilisation :

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Pour Diagnostiquer Analytics

Sert à déboguer les problèmes d’analyse : événements non envoyés et problèmes d’envoi groupé.

Exemple d’utilisation :

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Avec Le Débogage ADB Sans Fil

Le débogage sans fil demande parfois de redémarrer adb. Connectez-vous d’abord en USB, puis passez au sans-fil.

Exemple d’utilisation :

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Pendant Le Redémarrage De L’appareil

Redémarre l’appareil et suit les journaux, surtout pour les erreurs qui surviennent au démarrage.

Exemple d’utilisation (deux terminaux) :

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Pendant Le Redémarrage D’une Application

Redémarre une seule application et suit les journaux.

Exemple d’utilisation :

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Pendant L’installation D’une Application

Installe un APK et suit les journaux.

Exemple d’utilisation :

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Pendant La Désinstallation D’une Application

Désinstalle une application et suit les journaux.

Exemple d’utilisation :

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Avec Une Capture D’écran

Prend une capture d’écran et enregistre le journal au moment d’une erreur.

Exemple d’utilisation :

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Avec Un Enregistrement D’écran

Enregistre l’écran et le journal au moment d’une erreur. Pris en charge à partir d’Android 4.4.

Exemple d’utilisation :

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Avec Un Test Monkey

Teste une application en simulant des interactions utilisateur aléatoires. Très efficace avec les journaux.

Exemple d’utilisation :

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Avec Un Test UI Automator

Pour les tests d’automatisation de l’interface, afin de suivre les journaux pendant le test.

Exemple d’utilisation :

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Avec Un Test Espresso

Pour les tests Espresso, un sous-ensemble de l’automatisation de l’interface.

Exemple d’utilisation :

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Avec Un Test JUnit

Pour les tests unitaires. Ils s’exécutent généralement sur un ordinateur, mais aussi sur l’appareil.

Exemple d’utilisation :

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Avec StrictMode

Repère les problèmes de performance d’une application. Avertit quand un accès disque ou réseau est fait sur le thread principal.

Exemple d’utilisation :

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Avec Choreographer

Repère les images sautées. On regarde ici quand les animations saccadent.

Exemple d’utilisation :

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Avec Le Surdessin GPU

Repère les problèmes de surdessin (overdraw). Utilisé dans l’analyse des performances graphiques.

Exemple d’utilisation :

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Avec Le Thread De Rendu

Repère les problèmes du thread de rendu. Sur Android moderne, le dessin se fait sur ce thread.

Exemple d’utilisation :

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Avec Le Rendu GPU

Repère les problèmes de rendu GPU. On regarde ici une fois le profil de rendu GPU activé.

Exemple d’utilisation :

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Avec TextView

Repère les problèmes des vues de texte, comme les soucis de police et les erreurs de mesure.

Exemple d’utilisation :

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Avec RecyclerView

Repère les problèmes des vues de liste : performances de défilement et recyclage des vues.

Exemple d’utilisation :

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Avec ViewPager

Repère les problèmes de transitions entre pages.

Exemple d’utilisation :

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Avec Les Fragments

Repère les problèmes de cycle de vie des fragments.

Exemple d’utilisation :

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Avec Les Activités

Repère les problèmes de cycle de vie des activités.

Exemple d’utilisation :

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Avec Les Services

Repère les problèmes des services en arrière-plan.

Exemple d’utilisation :

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Avec WorkManager

Repère les problèmes de WorkManager, pour les tâches en arrière-plan.

Exemple d’utilisation :

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Avec AlarmManager

Repère les problèmes d’AlarmManager, pour les tâches planifiées.

Exemple d’utilisation :

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Avec SharedPreferences

Repère les problèmes de SharedPreferences (stockage de données simple).

Exemple d’utilisation :

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Avec SQLite

Repère les problèmes de SQLite (la base de données locale).

Exemple d’utilisation :

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Avec Les Opérations De Fichiers

Repère les problèmes de lecture et d’écriture de fichiers.

Exemple d’utilisation :

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Avec Les Opérations Réseau

Repère les problèmes de connexion réseau, comme HTTP, TCP et UDP.

Exemple d’utilisation :

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Avec SSL

Repère les problèmes de certificats SSL. On regarde ici quand les connexions sécurisées échouent.

Exemple d’utilisation :

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Avec JSON

Repère les problèmes d’analyse JSON.

Exemple d’utilisation :

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Avec XML

Repère les problèmes d’analyse XML.

Exemple d’utilisation :

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Avec Les Images

Repère les problèmes de chargement et de traitement d’images.

Exemple d’utilisation :

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Avec Les Médias

Repère les problèmes de lecture et d’enregistrement de médias.

Exemple d’utilisation :

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Avec ExoPlayer

Repère les problèmes d’ExoPlayer (bibliothèque de lecteur multimédia).

Exemple d’utilisation :

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Avec Chromecast

Repère les problèmes de Chromecast.

Exemple d’utilisation :

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Avec Les Publicités

Repère les problèmes des bibliothèques publicitaires.

Exemple d’utilisation :

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Avec La Facturation

Repère les problèmes de facturation Google Play.

Exemple d’utilisation :

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Avec Les Cartes

Repère les problèmes des bibliothèques de cartes.

Exemple d’utilisation :

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Avec L’authentification

Repère les problèmes de connexion, comme Google Sign-In et Firebase Auth.

Exemple d’utilisation :

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Avec Les Notifications

Repère les problèmes d’envoi et de réception de notifications.

Exemple d’utilisation :

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Avec Les Liens Profonds

Repère les problèmes de liens profonds (deep links), comme l’ouverture d’une application depuis une URL.

Exemple d’utilisation :

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Avec Les Intents

Repère les problèmes d’intents, pour passer d’une application à l’autre.

Exemple d’utilisation :

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Avec Les Broadcasts

Repère les problèmes de messages broadcast entre le système et les applications.

Exemple d’utilisation :

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Avec Les Content Providers

Repère les problèmes de content providers, pour partager des données entre applications.

Exemple d’utilisation :

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Avec JobScheduler

Repère les problèmes de JobScheduler, pour les tâches planifiées en arrière-plan.

Exemple d’utilisation :

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Avec Le Mode Doze

Repère les problèmes du mode Doze (veille), pour le travail en arrière-plan pendant que l’appareil dort.

Exemple d’utilisation :

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Avec L’optimisation De La Batterie

Repère les problèmes d’optimisation de la batterie, pour les restrictions en arrière-plan.

Exemple d’utilisation :

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Avec Les Fuites De Mémoire

Repère les fuites de mémoire. S’utilise avec des outils comme LeakCanary.

Exemple d’utilisation :

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Avec Les ANR

Repère les problèmes d’ANR (application qui ne répond pas).

Exemple d’utilisation :

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Avec Les Plantages

Repère les plantages.

Exemple d’utilisation :

```
adb logcat -b crash
```

---

135- Adb Logcat Avec Les Plantages Natifs

Repère les plantages natifs (C/C++/Rust).

Exemple d’utilisation :

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Avec L’analyse Des Tombstones

Analyse les enregistrements de plantages natifs.

Exemple d’utilisation :

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Avec L’analyse Systrace

Analyse les enregistrements de traçage système.

Exemple d’utilisation :

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Avec L’analyse Perfetto

L’outil moderne de traçage système, plus complet que logcat.

Exemple d’utilisation :

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Avec L’analyse Atrace

Pour le traçage au niveau système ; l’ancienne forme de Perfetto.

Exemple d’utilisation :

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Avec L’analyse Traceview

Pour le profilage au niveau des méthodes.

Exemple d’utilisation :

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Avec Le Traçage De Méthode

Suit une méthode précise.

Exemple d’utilisation :

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Avec Un Vidage Du Tas

Réalise un vidage de la mémoire. Très utile pour l’analyse de la mémoire.

Exemple d’utilisation :

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Avec Le Journal Du GC

Suit les événements du ramasse-miettes (garbage collection).

Exemple d’utilisation :

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Avec Le Débogage JNI

Repère les problèmes de JNI (Java Native Interface).

Exemple d’utilisation :

```
adb logcat -s JNI
```

---

145- Adb Logcat Avec Le Débogage ART

Repère les problèmes d’ART (Android Runtime).

Exemple d’utilisation :

```
adb logcat -s Art
```

---

146- Adb Logcat Avec Le Débogage Dalvik

Repère les problèmes de la machine virtuelle Dalvik. Utilisé sur les anciennes versions d’Android.

Exemple d’utilisation :

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat Et Le Problème Chatty

Quand logcat écrit trop de journaux, il donne un avertissement « chatty » et saute des lignes. La solution : un filtre ou un tampon plus grand.

Exemple d’utilisation (agrandir le tampon) :

```
adb logcat -G 16M
```

Exemple d’utilisation (utiliser un filtre) :

```
adb logcat -s MyApp
```

---

148- Adb Logcat Avec Une Sortie En Couleur

Dans les longs flux de journaux, la couleur repose les yeux et rend l’erreur visible tout de suite.

Exemple d’utilisation :

```
adb logcat -v color
```

Exemple d’utilisation (avec time) :

```
adb logcat -v time,color
```

---

149- Adb Logcat Avec Un Script Bash

Sert à écrire un script qui collecte automatiquement les journaux. On peut bâtir dessus un système de rapports de bugs.

Exemple de script :

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat En Résumé

Comme le montre cette liste, logcat n’est pas une commande unique mais un ensemble de commandes. Associé aux filtres, aux formats, aux tampons, aux fichiers et à d’autres outils, il devient l’arme la plus puissante du débogage. Si vous savez quelle combinaison convient à quelle situation, trouver un bug prend quelques minutes. Retenez : la première règle d’une session de débogage est toujours la même ; videz d’abord avec adb logcat -c, reproduisez ensuite le bug, puis observez avec un logcat filtré.

@@ Connexion Android Debug Bridge
@: Commandes De Connexion ADB Sans Fil
1- Activer Le Débogage Sans Fil

Il faut d’abord activer les options pour les développeurs sur l’appareil. Pour cela, touchez 7 fois Paramètres → À propos du téléphone → Numéro de build. Activez ensuite Paramètres → Système → Options pour les développeurs → Débogage sans fil. Sans cette étape, aucune des commandes adb pair et adb connect ne fonctionne.

Exemple de chemin :

```
Paramètres → À propos du téléphone → Numéro de build (toucher 7 fois)
Paramètres → Système → Options pour les développeurs → Débogage sans fil (activé)
```

---

2- Trouver L’adresse IP Et Le Port De L’appareil

Une fois le débogage sans fil activé, ouvrez l’option « Associer l’appareil avec un code d’association ». L’adresse IP de l’appareil et un numéro de port aléatoire y apparaissent, par exemple 192.168.1.42:37123. Ce port ne sert qu’à l’association et change ensuite. Un code d’association à 6 chiffres figure aussi sur le même écran.

Exemple d’affichage :

```
Adresse IP et port
192.168.1.42:37123

Code d’association Wi-Fi
482915
```

---

3- La Commande Adb Pair

Cette commande associe l’appareil. Elle s’exécute sur un ordinateur ou dans Termux. Saisissez l’adresse IP et le port d’association affichés sur l’appareil. À l’exécution, le terminal demande Enter pairing code: et attend le code à 6 chiffres. Si le code est bon, le message Successfully paired s’affiche.

Exemple d’utilisation :

```
adb pair 192.168.1.42:37123
```

Exemple de sortie :

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Avec un mauvais code, elle échoue ainsi :

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- D’autres Façons De Donner Le Code À Adb Pair

Il n’est pas obligatoire de saisir le code à la main à chaque fois. Certaines versions l’acceptent directement en paramètre, même si, pour des raisons de sécurité, ce n’est généralement pas pris en charge. On peut tout de même essayer.

Exemple d’utilisation (sur une ligne) :

```
adb pair 192.168.1.42:37123 482915
```

Exemple d’utilisation (avec un pipe) :

```
echo "482915" | adb pair 192.168.1.42:37123
```

Ces méthodes varient selon les versions. Le plus fiable est d’exécuter la commande et de saisir le code à la main.

---

5- Points D’attention Avec Adb Pair

Le port d’association et le port de connexion sont différents. Le port d’association est généralement aléatoire et élevé, le port de connexion généralement 5555. L’association ne se fait qu’une fois ; adb connect suffit pour se reconnecter au même appareil. Mais si vous réinitialisez l’appareil ou désactivez le débogage sans fil, l’association est perdue et adb pair redevient nécessaire.

Exemple de comparaison :

```
Association : adb pair 192.168.1.42:37123   → demande un code, une seule fois
Connexion :   adb connect 192.168.1.42:5555  → sans code, à chaque fois
```

---

6- La Commande Adb Connect

Cette commande se connecte à un appareil associé. Saisissez l’adresse IP et le port affichés sur l’écran principal du débogage sans fil. Le port est généralement 5555, mais peut parfois différer. Si la connexion réussit, le message Connected to s’affiche.

Exemple d’utilisation :

```
adb connect 192.168.1.42:5555
```

Exemple de sortie :

```
Connected to 192.168.1.42:5555
```

S’il est déjà connecté, il prévient ainsi :

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Sans Port

Dans certaines versions, 5555 est essayé par défaut quand aucun port n’est indiqué. Ce comportement n’est pas le même sur tous les appareils ; écrire le port explicitement reste donc le plus sûr.

Exemple d’utilisation (port par défaut) :

```
adb connect 192.168.1.42
```

Exemple d’utilisation (port explicite) :

```
adb connect 192.168.1.42:5555
```

---

8- Vérifier La Connexion

Une fois connecté, la commande adb devices montre si l’appareil est vraiment connecté. L’adresse IP de l’appareil doit apparaître dans la liste avec device à côté. S’il est écrit offline, la connexion a été établie mais n’est pas encore stable ; réessayez peu après.

Exemple d’utilisation :

```
adb devices
```

Exemple de sortie :

```
List of devices attached
192.168.1.42:5555   device
```

Exemple de sortie (non connecté) :

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- La Commande Adb Disconnect

Met fin à la connexion. La connexion adb se termine même si l’appareil reste sur le même réseau. Pour se reconnecter, relancez adb connect ; aucune association n’est nécessaire.

Exemple d’utilisation (un appareil) :

```
adb disconnect 192.168.1.42:5555
```

Exemple de sortie :

```
Disconnected 192.168.1.42:5555
```

Exemple d’utilisation (tous les appareils) :

```
adb disconnect
```

Exemple de sortie :

```
Disconnected everything
```

---

10- La Commande Adb Tcpip

Fait passer en mode sans fil un appareil connecté en USB. Cette commande fait écouter l’appareil sur le port indiqué, généralement 5555. Cette méthode est plus courante sur les versions antérieures à Android 11, qui n’ont pas adb pair.

Exemple d’utilisation :

```
adb tcpip 5555
```

Exemple de sortie :

```
Restarting in TCP mode port: 5555
```

Débranchez ensuite le câble USB et exécutez cette commande :

```
adb connect 192.168.1.42:5555
```

---

11- La Commande Adb Usb

Revient du mode sans fil au mode USB, quand on veut reconnecter l’appareil avec un câble.

Exemple d’utilisation :

```
adb usb
```

Exemple de sortie :

```
Restarting in USB mode
```

---

12- Adb Kill-Server Et Start-Server

Le serveur adb se bloque parfois ou refuse les connexions. Il faut alors l’arrêter puis le redémarrer. Cela réinitialise toutes les connexions et repart sur une base propre.

Exemple d’utilisation :

```
adb kill-server
adb start-server
```

Exemple de sortie :

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Puis reconnectez-vous :

```
adb connect 192.168.1.42:5555
```

---

13- Être Sur Le Même Réseau

Pour l’adb sans fil, l’appareil et l’ordinateur ou le téléphone doivent être sur le même réseau Wi-Fi. Sur des réseaux différents, aucune connexion ne s’établit. Un réseau invité, la séparation 5 GHz et 2,4 GHz, un VPN ou un pare-feu posent parfois problème. La solution simple : connectez les deux appareils au même routeur et coupez les VPN.

Exemple de vérification :

```
Appareil : Paramètres → Wi-Fi → Réseau connecté (ex. Home_Wifi)
Termux : Paramètres → Wi-Fi → Réseau connecté (ex. Home_Wifi)
```

---

14- Trouver L’adresse IP À La Main

Pour trouver l’adresse IP de l’appareil, suivez Paramètres → Wi-Fi → Réseau connecté → Détails du réseau. Ou regardez dans Termux avec la commande ci-dessous. Mais l’adresse IP peut changer avec le temps ; le plus sûr est donc de vérifier à chaque fois l’écran du débogage sans fil.

Exemple d’utilisation (Termux) :

```
ifconfig wlan0
```

Exemple d’utilisation (sur l’appareil) :

```
Paramètres → Wi-Fi → Réseau connecté → Adresse IP
```

---

15- Se Connecter À Soi-Même Depuis Termux Sur Le Même Appareil

Si Termux tourne sur le même appareil, adb est déjà local, mais on peut quand même se connecter via localhost avec adb connect. On utilise généralement 127.0.0.1:5555. Certains appareils le refusent ; on utilise alors l’adresse IP locale de l’appareil.

Exemple d’utilisation (localhost) :

```
adb connect 127.0.0.1:5555
```

Exemple d’utilisation (son IP locale) :

```
adb connect 192.168.1.42:5555
```

---

16- Exécuter Des Commandes Avec Plusieurs Appareils Connectés

Quand plusieurs appareils sont connectés, choisissez l’appareil avec le paramètre -s dans chaque commande. Sinon adb échoue avec error: more than one device.

Exemple d’utilisation :

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Quand La Connexion Se Coupe

Si le signal Wi-Fi faiblit, si l’appareil se met en veille ou si le réseau change, la connexion se coupe. L’appareil disparaît alors de la liste adb devices ou apparaît offline. La solution : relancer adb connect. L’association reste généralement valable, adb pair n’est pas nécessaire.

Exemple de reprise :

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Après Un Redémarrage De L’appareil

Au redémarrage, le débogage sans fil se désactive généralement. Sur certains appareils il reste activé, mais l’association est réinitialisée. Une fois l’appareil démarré, ouvrez l’écran du débogage sans fil, prenez le nouveau port et le nouveau code, relancez adb pair puis adb connect.

Exemple de déroulement :

```
Appareil redémarré → activer le débogage sans fil
Nouveau port : 42157, nouveau code : 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Erreurs Fréquentes Lors De L’association

Failed: Wrong password → le code a été mal saisi.
Failed to connect: Connection refused → le port est faux ou l’appareil n’écoute pas.
Failed: Pairing rejected → l’appareil a refusé l’association ; renouvelez le code.
Timeout → problème réseau ; vérifiez que vous êtes sur le même réseau.

Exemple d’erreur et de solution :

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Solution : le code sur l’appareil a changé ; prenez le nouveau code et réessayez.

---

20- Désactiver L’adb Sans Fil

Désactiver le débogage sans fil quand vous avez terminé est une bonne idée pour la sécurité. Vous pouvez le couper dans les paramètres ou passer en mode USB avec adb.

Exemple d’utilisation :

```
adb usb
adb disconnect
```

Exemple d’utilisation (depuis les paramètres) :

```
Paramètres → Système → Options pour les développeurs → Débogage sans fil (désactivé)
```

---

21- Différences Selon La Version D’Android

Android 11 et plus : l’association avec adb pair est obligatoire et un port aléatoire est utilisé.
Android 10 et moins : on passe en mode sans fil avec adb tcpip 5555 ; il n’y a pas d’association.

Exemple de comparaison :

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Notes De Sécurité

Tant que le débogage sans fil est activé, une personne malveillante sur le même réseau pourrait tenter d’accéder à l’appareil. Gardez donc cette fonction désactivée sur les réseaux Wi-Fi auxquels vous ne faites pas confiance. Le code d’association existe pour la sécurité, mais la prudence reste de mise. Désactivez-le quand vous avez terminé.

Exemple de précaution :

```
Quand vous ne l’utilisez pas : Paramètres → Options pour les développeurs → Débogage sans fil (désactivé)
Sur un réseau sûr : ne l’activer qu’à la maison ou au bureau
```

---

23- Tout Le Déroulement En Un Seul Endroit

Le script ci-dessous montre toutes les étapes, du départ à la connexion. Adaptez simplement l’IP et le port à votre appareil.

Exemple de déroulement :

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Résultat attendu :

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Un Script Raccourci Pour Une Connexion Durable

Si vous vous connectez souvent, écrivez un script et connectez-vous à chaque fois en une seule commande. Dans Termux, vous pouvez par exemple ajouter une fonction au fichier ~/.bashrc.

Exemple de fonction :

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Saisissez l'IP:port et le code affichés sur l'appareil :"
    read -p "Adresse : " ADRES
    adb pair "$ADRES"
    read -p "Port de connexion (ex. 5555) : " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Utilisation :

```
wifi_adb
```

---

25- Note Finale

L’adb sans fil permet de déboguer sans câble USB et se révèle particulièrement pratique quand on travaille dans Termux sur le téléphone lui-même. Les deux commandes de base sont adb pair et adb connect ; la première associe une fois, la seconde sert à chaque connexion. Les autres (adb devices, adb disconnect, adb tcpip, adb kill-server) existent pour soutenir ces deux commandes et régler les problèmes. N’oubliez pas la règle du même réseau, désactivez-le quand vous ne l’utilisez pas, et redémarrez le serveur en cas de souci. C’est tout.
"#;

const ITALIAN: &str = r#"@@ Comandi Logcat
1- Adb Logcat

Invia in diretta al terminale tutti i log del dispositivo. Dal momento in cui il comando parte, ogni riga di log prodotta da sistema, app, servizi e kernel compare sullo schermo. Per fermarlo premi Ctrl + C sulla tastiera. Questo comando è la base del debug; ma senza filtro può far scorrere centinaia di righe al secondo e trovare ciò che serve diventa difficile.

Esempio d’uso:

```
adb logcat
```

Esempio di output:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Mostra i log già accumulati e poi il comando termina. Invece di un flusso continuo, scatta un’istantanea di quel momento. Si usa per preparare una segnalazione di bug, salvare lo stato dopo un crash o prendere i log una sola volta da uno script.

Esempio d’uso:

```
adb logcat -d
```

Esempio d’uso (salvataggio su file):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Questo comando è molto usato negli script che inviano automaticamente rapporti a un server, perché termina da solo e non blocca mai lo script.

---

3- Adb Logcat -C

Svuota tutti i buffer di log e cancella tutto ciò che è stato accumulato. Se lo esegui prima di una nuova sessione di debug, vedrai solo i log prodotti da quel momento e il vecchio rumore non ti confonderà. Quando provi a riprodurre un bug, di solito è il primo passo.

Esempio d’uso:

```
adb logcat -c
adb logcat
```

Eseguendo in ordine i due comandi qui sopra si ottiene un flusso di log perfettamente pulito. Compare solo ciò che accade dopo lo svuotamento.

---

4- Adb Logcat -G

Mostra la dimensione dei buffer di log attuali e quanto sono pieni. Elenca, riga per riga, il riempimento di ogni buffer, la capacità totale e la quantità usata. Se i log scorrono velocissimi e le voci vecchie vengono scartate (si dice «chatty»), guarda prima qui.

Esempio d’uso:

```
adb logcat -g
```

Esempio di output:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Dimensione

Cambia la dimensione del buffer di log. Ad esempio puoi portare un buffer da 256 KiB a 16 MiB. Così si conservano più log e le voci vecchie vengono scartate più tardi. Molto utile nei test lunghi o nel debug di un’app molto rumorosa. L’impostazione si azzera al riavvio del dispositivo.

Esempio d’uso:

```
adb logcat -G 16M
```

Esempio d’uso (per ridurlo):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F File

Scrive i log nel file indicato invece di stamparli nel terminale. Il terminale resta pulito e la registrazione si conserva. Il percorso del file è sul dispositivo, non su un computer. In un terminale mobile (Termux) di solito si scrive sotto /Sdcard/.

Esempio d’uso:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Esempio d’uso (aggiungendo al file):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

Nell’esempio qui sopra si apre un nuovo file quando il file raggiunge 4 MiB, e in totale si conservano 10 file.

---

7- Adb Logcat -B Main

Osserva solo il buffer principale delle app. Qui finiscono i log che le app scrivono con chiamate come Log.d, Log.i e Log.e. È già osservato per impostazione predefinita, ma lo si indica esplicitamente quando si vuole solo questo buffer. È il posto dove si guarda di più sviluppando un’app.

Esempio d’uso:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Osserva i log dei servizi di sistema. Qui finiscono i log di servizi come ActivityManager, WindowManager, PackageManager e PowerManager. Serve a vedere eventi come l’avvio delle app, la concessione dei permessi e il collegamento dei servizi. Di solito si osserva insieme a main.

Esempio d’uso:

```
adb logcat -b system
```

Esempio di output:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Osserva solo i log dei crash. Qui finiscono i blocchi FATAL EXCEPTION scritti quando un’app viene chiusa forzatamente. Nel debug di un crash, guardare prima questo buffer è la via più rapida, perché lo stack trace è proprio qui.

Esempio d’uso:

```
adb logcat -b crash
```

Esempio di output:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Osserva i log di radio, telefonia, segnale e connessione dati. Utile con la SIM, la registrazione alla rete, la connessione 4G/5G, gli SMS o le chiamate. Di solito non serve nello sviluppo di app comune.

Esempio d’uso:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Osserva i log binari degli eventi di sistema. Questo buffer non contiene testo normale ma brevi record codificati lasciati dalle chiamate Log.event. Compaiono con tag come am_proc_start e am_activity_launch. Si usa nell’analisi delle prestazioni.

Esempio d’uso:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Osserva i log del kernel. Mostra i messaggi scritti dal kernel Linux, simili di solito all’output del comando dmesg. Qui ci sono errori dei driver, problemi di memoria e informazioni su temperatura e gestione energetica. Può richiedere root.

Esempio d’uso:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Osserva tutti i buffer insieme. Main, system, crash, radio, events e kernel scorrono mescolati. Utile quando vuoi vedere tutto, ma l’output diventa molto affollato, quindi di solito si abbina a un filtro.

Esempio d’uso:

```
adb logcat -b all
```

Esempio d’uso (con filtro):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Più Buffer

Osserva insieme più buffer separati da virgole, ad esempio per vedere insieme i log dell’app e dei crash. Scegliendo solo i canali necessari, restringi l’output.

Esempio d’uso:

```
adb logcat -b main,crash
```

Esempio d’uso (tre canali):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Il formato di output breve predefinito. Ogni riga inizia solo con la priorità e il tag. Buono per un’occhiata veloce, ma senza timestamp è difficile ordinare gli eventi.

Esempio d’uso:

```
adb logcat -v brief
```

Esempio di output:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Aggiunge data e ora. Molto più utile del formato breve, perché vedi quando è avvenuto un evento. Sotto il secondo compaiono i millisecondi.

Esempio d’uso:

```
adb logcat -v time
```

Esempio di output:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

Il formato di output più consigliato. Contiene data, ora, priorità, tag, PID (identificativo del processo) e TID (identificativo del thread). Nelle app con più thread è indispensabile per capire quale thread ha fatto cosa. È anche il formato usato di default da Android Studio.

Esempio d’uso:

```
adb logcat -v threadtime
```

Esempio di output:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Dà un output dettagliato con tutti i campi di metadati e una riga vuota tra le voci. Nelle sessioni lunghe mostra ogni log come un blocco, ma occupa spazio.

Esempio d’uso:

```
adb logcat -v long
```

Esempio di output:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colora i log in base alla priorità. Gli errori diventano rossi, gli avvisi arancioni, le info verdi e il debug blu. In un flusso lungo l’occhio va subito all’errore. Il terminale deve supportare i colori.

Esempio d’uso:

```
adb logcat -v color
```

Esempio d’uso (insieme a time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Aggiunge l’anno al formato orario. Si usa per registrazioni lunghe o archivi che coprono più anni. Di solito non serve nello sviluppo comune.

Esempio d’uso:

```
adb logcat -v year
```

Esempio di output:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Mostra l’identificativo utente (UID) dell’app che ha prodotto il log. Utile quando più app usano lo stesso tag. È un modificatore, quindi si combina con gli altri formati.

Esempio d’uso:

```
adb logcat -v uid
```

Esempio di output:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filtro Per Tag E Priorità

Mostra i log di un certo tag a partire da una certa priorità. Le priorità sono: V (Verbose) la più bassa, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) e S (Silent) la più alta. Questo modo di filtrare è la funzione più potente di logcat.

Esempio d’uso (solo errori):

```
adb logcat *:E
```

Esempio d’uso (avvisi e superiori):

```
adb logcat *:W
```

Esempio d’uso (lista consentita):

```
adb logcat ActivityManager:I MyApp:D *:S
```

L’ultimo esempio mostra ActivityManager da info in su e MyApp da debug in su, e silenzia tutti gli altri tag. In alcune shell il carattere * è speciale, quindi va messo tra virgolette:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Tag

Una scorciatoia del filtro silenzioso. Mostra solo il tag indicato e silenzia tutto il resto. Il modo più pratico per concentrarsi sul proprio tag mentre si sviluppa.

Esempio d’uso:

```
adb logcat -s MyApp
```

Esempio d’uso (più tag):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

Nell’esempio qui sopra scorrono solo i log di tre tag; tutto il resto resta in silenzio.

---

24- Adb Logcat -S Con Priorità

Serve a indicare insieme tag e priorità, ad esempio per vedere solo avvisi e superiori del tag MyApp.

Esempio d’uso:

```
adb logcat MyApp:W *:S
```

Questo comando mostra solo i log warning, error e fatal con il tag MyApp.

---

25- Adb Logcat Filtro --Pid

Mostra solo i log di un certo identificativo di processo (PID). Per conoscere il PID di un’app si usa prima adb shell pidof, poi si filtra con quel PID. Perfetto per isolare tutti i log di una sessione.

Esempio d’uso (ottenere il PID):

```
adb shell pidof com.example.app
```

Esempio di output:

```
5678
```

Esempio d’uso (filtrare):

```
adb logcat --pid=5678
```

Esempio d’uso (in una riga):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Ora

Mostra i log a partire dal timestamp indicato, cioè «dammi quello che viene dopo questo momento». Si scrive con la T maiuscola; la t minuscola significa altro.

Esempio d’uso:

```
adb logcat -T "09-21 14:00:00.000"
```

Esempio d’uso (da oggi in poi):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Numero Di Righe

Con la t minuscola mostra il numero indicato di righe più recenti e poi il comando termina. Si comporta come -d ma dà solo le ultime N righe. Ideale per un’occhiata veloce.

Esempio d’uso (ultime 100 righe):

```
adb logcat -t 100
```

Esempio d’uso (ultime 500 righe e un filtro):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Cerca nel contenuto con un’espressione regolare (regex) e mostra le righe corrispondenti. Molto potente per ricerche testuali rapide; soprattutto con più parole insieme è più veloce del filtro.

Esempio d’uso (una parola):

```
adb logcat -e "Login"
```

Esempio d’uso (più parole):

```
adb logcat -e "Login|Error|Crash"
```

Esempio d’uso (con il nome del pacchetto):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Output Binario

Scrive i log in formato binario. Non è leggibile da una persona, ma un altro logcat può rileggerlo o altri strumenti possono elaborarlo. Si usa nei sistemi di analisi automatica.

Esempio d’uso:

```
adb logcat -B > Ham_Log.Bin
```

In seguito quel file si legge così:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Statistiche

Mostra le statistiche dei log: quanti log ha prodotto ogni tag e quali tag hanno fatto scattare l’avviso «chatty». Se pensi che un’app stia inondando il log, lo confermi con questo comando.

Esempio d’uso:

```
adb logcat -S
```

Esempio di output:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R E -N Insieme

Per i log a rotazione. -r fissa la dimensione di ogni file e -n quanti file conservare. Utile quando un’app gira a lungo e si vuole registrare di continuo. I file vecchi vengono cancellati e ne vengono aperti di nuovi.

Esempio d’uso:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

Nell’esempio qui sopra si apre un nuovo file ogni volta che uno raggiunge 4 MiB e in totale si conservano 10 file. Così lo spazio non si riempie mai all’infinito.

---

32- Adb Logcat -P Script Di Filtro

Indica uno script di filtro personalizzato per logcat. È il formato di filtro proprio di Android e lo usano di solito gli sviluppatori di sistema. Gli sviluppatori di app lo usano raramente.

Esempio d’uso:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Timestamp

Un’opzione che in alcuni formati aggiunge un timestamp all’inizio di ogni riga. Il comportamento cambia tra le versioni di Android. Di solito fa lo stesso di -v time, ma in un’altra forma.

Esempio d’uso:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Mostra ogni riga in formato printf, cioè priorità (PID) tag: messaggio. Serve per la compatibilità con strumenti vecchi, ma su Android moderno serve di rado.

Esempio d’uso:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Mostra solo il messaggio grezzo, senza metadati. Si scrive solo il log, senza tag, priorità né timestamp. Utile per passare l’output a un altro programma.

Esempio d’uso:

```
adb logcat -v raw
```

Esempio di output:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Scrive solo il tag all’inizio di ogni riga, poi il resto. Priorità e PID sono nascosti. Comodo quando ci si concentra su un tag.

Esempio d’uso:

```
adb logcat -v tag
```

Esempio di output:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Scrive prima il PID e poi il tag all’inizio di ogni riga. Comodo per l’analisi per processo; si vede subito quale processo ha scritto cosa.

Esempio d’uso:

```
adb logcat -v process
```

Esempio di output:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Scrive solo l’identificativo del thread (TID) all’inizio di ogni riga, poi il resto. Nelle analisi con più thread mostra quale thread è attivo. Un formato breve.

Esempio d’uso:

```
adb logcat -v thread
```

Esempio di output:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D E -T Insieme

Prende i log di un intervallo di tempo, ad esempio quelli dell’ultima ora, per salvarli in un file.

Esempio d’uso:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Esempio d’uso (in un file):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C E -D In Sequenza

Un flusso che prima svuota e poi raccoglie. Ad esempio svuotare prima di un test e prendere i log alla fine del test.

Esempio d’uso:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Salvato Su File E Portato Sul Computer

Comodo per registrare a lungo in un terminale mobile. Prima si scrive su un file del dispositivo, poi lo si porta sul computer con adb pull.

Esempio d’uso (salvare sul dispositivo):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Esempio d’uso (portare sul computer):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Con Grep

Si usa in un terminale mobile (Termux) per cercare testo velocemente con grep. Può essere più flessibile del filtro regex, perché hai tutte le opzioni di grep.

Esempio d’uso:

```
adb logcat | grep "Hata"
```

Esempio d’uso (senza distinguere maiuscole):

```
adb logcat | grep -I "error"
```

Esempio d’uso (con i numeri di riga):

```
adb logcat | grep -N "Exception"
```

Esempio d’uso (più parole):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Con Tail

Riduce il flusso in diretta alle sue ultime righe, quando vuoi vedere solo la fine invece di tutto il log.

Esempio d’uso:

```
adb logcat | tail -100
```

Esempio d’uso (seguire di continuo):

```
adb logcat | tail -F
```

---

44- Adb Logcat Con Head

Mostra le prime righe. Buono per un’occhiata veloce a una registrazione lunga.

Esempio d’uso:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Con Wc

Mostra il numero totale di righe, ad esempio per misurare quanti log ha prodotto un test.

Esempio d’uso:

```
adb logcat -d | wc -l
```

Esempio d’uso (contare solo gli errori):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Con Sort

Ordina i log in ordine alfabetico o numerico. Si usa soprattutto durante l’analisi.

Esempio d’uso:

```
adb logcat -d | sort
```

Esempio d’uso (ordine inverso):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Con Uniq

Rimuove le righe ripetute. Se un’app scrive di continuo lo stesso errore, questo comando semplifica l’output.

Esempio d’uso:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

L’esempio qui sopra conta e mostra le 20 righe più ripetute. Molto utile nel debug.

---

48- Adb Logcat Con Awk

Estrae colonne precise o lavora sul testo, ad esempio la colonna PID nel formato threadtime.

Esempio d’uso (solo la colonna PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Esempio d’uso (solo tag e messaggio):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Con Xargs

Esegue un comando per ogni riga, ad esempio inviare una notifica per ogni riga di errore. Si usa soprattutto nell’automazione.

Esempio d’uso:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Con Sed

Sostituisce il testo. Ideale per cambiare certi schemi nell’output.

Esempio d’uso:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

L’esempio qui sopra scrive Uygulama al posto di ogni MyApp.

---

51- Adb Logcat Con Tee

Mostra a schermo e scrive su un file nello stesso momento. Osservazione dal vivo e registrazione avvengono insieme.

Esempio d’uso:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Con Watch

Prende i log a intervalli fissi, ad esempio un riepilogo ogni 5 secondi.

Esempio d’uso:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Con Timeout

Ferma logcat dopo un tempo stabilito. Molto utile nei test automatici.

Esempio d’uso:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Con Alias

Accorcia i comandi che usi spesso. In Termux si aggiunge al file .bashrc o .zshrc.

Esempio d’uso (definire un alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Poi basta chiamarlo così:

```
lce
```

---

55- Adb Logcat Con Più Dispositivi

Punta a un dispositivo preciso quando ne sono collegati diversi. Prima si elencano i dispositivi con adb devices, poi se ne sceglie uno con -s.

Esempio d’uso:

```
adb devices
```

Esempio di output:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Esempio d’uso (un dispositivo preciso):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Con Il Debug Wireless

Per il debug wireless. Quando il dispositivo è sulla stessa rete Wi-Fi, ci si collega senza cavo. Supportato da Android 11.

Esempio d’uso (attiva il debug wireless sul dispositivo, poi):

```
adb pair 192.168.1.42:37123
```

Esempio d’uso (connessione):

```
adb connect 192.168.1.42:5555
```

Esempio d’uso (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Con Bugreport

Crea un rapporto dettagliato di tutto il sistema. Oltre a logcat vengono eseguiti molti strumenti come dumpsys, dmesg e top, e tutto finisce in un unico zip. Il modo di debug più completo.

Esempio d’uso:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Esempio d’uso (portare sul computer):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Con Dumpsys

Mostra, accanto a logcat, lo stato attuale dei servizi di sistema, ad esempio cosa sta facendo ActivityManager in quel momento.

Esempio d’uso:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Con Le Tracce ANR

Prende gli stack trace quando un’app non risponde (ANR). I record ANR si trovano nel file /Data/Anr/Traces.Txt.

Esempio d’uso:

```
adb shell cat /Data/Anr/Traces.Txt
```

Esempio d’uso (serve root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Con I Tombstone

Per i crash nativi. Nella cartella /Data/Tombstones/ viene creato un file per ogni crash. Si guarda qui quando il codice C++ o Rust ha un errore.

Esempio d’uso:

```
adb shell ls /Data/Tombstones/
```

Esempio d’uso (l’ultimo tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Con Systrace

Per il tracciamento delle prestazioni di tutto il sistema. Raccoglie dati di tracciamento a livello di kernel, non solo log. Si usa insieme a Perfetto.

Esempio d’uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Con Layout Inspector

Analizza il layout di un’app sullo schermo. Si usa con Layout Inspector, ma i log correlati si possono seguire anche con logcat.

Esempio d’uso:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Con Memory Profiler

Osserva l’uso della memoria. Memory Profiler di solito è in Android Studio, ma i log legati alla memoria si vedono anche in logcat.

Esempio d’uso:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Con CPU Profiler

Osserva l’uso della CPU. Per una misura semplice va bene anche il comando top.

Esempio d’uso:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Con Network Profiler

Osserva il traffico di rete. Librerie come OkHttp e Retrofit scrivono in logcat, ma un’analisi completa richiede Network Profiler.

Esempio d’uso:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Con GPU Profiler

Osserva l’uso della GPU. Per una misura semplice va bene anche il comando gfxinfo.

Esempio d’uso:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Con Battery Historian

Osserva il consumo della batteria. Si può guardare anche con dumpsys battery.

Esempio d’uso:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Per Diagnosticare Il Wi-Fi

Serve a fare il debug dei problemi Wi-Fi, come cadute di connessione o segnale debole.

Esempio d’uso:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Per Diagnosticare Il Bluetooth

Serve a fare il debug dei problemi Bluetooth, come errori di associazione o disconnessioni.

Esempio d’uso:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Per Diagnosticare I Sensori

Serve a fare il debug dei problemi dei sensori come accelerometro, giroscopio e magnetometro.

Esempio d’uso:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Per Diagnosticare La Fotocamera

Serve a fare il debug dei problemi della fotocamera, come scattare foto o registrare video.

Esempio d’uso:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Per Diagnosticare L’audio

Serve a fare il debug dei problemi audio, come nessun suono o un microfono che non funziona.

Esempio d’uso:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Per Diagnosticare La Posizione

Serve a fare il debug dei problemi di posizione, come nessun segnale GPS o una posizione che non si aggiorna.

Esempio d’uso:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Per Diagnosticare Le Notifiche

Serve a fare il debug dei problemi delle notifiche, come notifiche che non compaiono o arrivano al momento sbagliato.

Esempio d’uso:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Per Diagnosticare I Permessi

Serve a fare il debug dei problemi dei permessi, come un permesso non concesso o rifiutato.

Esempio d’uso:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Per Diagnosticare WebView

Serve a fare il debug dei problemi di WebView, come una pagina che non si carica o un errore JavaScript.

Esempio d’uso:

```
adb logcat -s Chromium WebView
```

Esempio d’uso (più dettagli):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Per Diagnosticare OkHttp

Serve a fare il debug dei problemi di OkHttp: richieste di rete, risposte ed errori.

Esempio d’uso:

```
adb logcat -s Okhttp
```

Esempio d’uso (tutti i livelli):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Per Diagnosticare Retrofit

Serve a fare il debug dei problemi di Retrofit. Poiché di solito si basa su OkHttp, si guardano i log di quest’ultimo.

Esempio d’uso:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Per Diagnosticare Glide

Serve a fare il debug dei problemi di Glide (libreria di caricamento immagini), come immagini che non si caricano o problemi di memoria.

Esempio d’uso:

```
adb logcat -s Glide
```

---

80- Adb Logcat Per Diagnosticare Picasso

Serve a fare il debug dei problemi di Picasso (libreria di caricamento immagini).

Esempio d’uso:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Per Diagnosticare Room

Serve a fare il debug dei problemi di Room (libreria di database): errori di query e problemi di schema.

Esempio d’uso:

```
adb logcat -s Room
```

---

82- Adb Logcat Per Diagnosticare Firebase

Serve a fare il debug dei problemi di Firebase, per moduli come Crashlytics, Analytics e Messaging.

Esempio d’uso:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Per Diagnosticare Crashlytics

Serve a fare il debug dei problemi di Firebase Crashlytics.

Esempio d’uso:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Per Diagnosticare Analytics

Serve a fare il debug dei problemi di analisi: eventi non inviati e problemi di invio in blocco.

Esempio d’uso:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Con Il Debug ADB Wireless

Il debug wireless a volte richiede di riavviare adb. Collegati prima via USB, poi passa al wireless.

Esempio d’uso:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Durante Il Riavvio Del Dispositivo

Riavvia il dispositivo e osserva i log, soprattutto per gli errori che avvengono all’avvio.

Esempio d’uso (due terminali):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Durante Il Riavvio Di Un’app

Riavvia una sola app e osserva i log.

Esempio d’uso:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Durante L’installazione Di Un’app

Installa un APK e osserva i log.

Esempio d’uso:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Durante La Disinstallazione Di Un’app

Disinstalla un’app e osserva i log.

Esempio d’uso:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Con Uno Screenshot

Scatta uno screenshot e salva il log nel momento di un errore.

Esempio d’uso:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Con Una Registrazione Dello Schermo

Registra lo schermo e salva il log nel momento di un errore. Supportato da Android 4.4.

Esempio d’uso:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Con Un Test Monkey

Testa un’app simulando interazioni casuali dell’utente. Molto efficace insieme ai log.

Esempio d’uso:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Con Un Test UI Automator

Per i test di automazione dell’interfaccia, osservando i log durante il test.

Esempio d’uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Con Un Test Espresso

Per i test Espresso, un sottoinsieme dell’automazione dell’interfaccia.

Esempio d’uso:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Con Un Test JUnit

Per i test unitari. Di solito girano su un computer, ma possono girare anche sul dispositivo.

Esempio d’uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Con StrictMode

Individua i problemi di prestazioni di un’app. Avvisa quando si accede al disco o alla rete sul thread principale.

Esempio d’uso:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Con Choreographer

Individua i frame saltati. Si guarda qui quando le animazioni scattano.

Esempio d’uso:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Con L’overdraw Della GPU

Individua i problemi di overdraw. Si usa nell’analisi delle prestazioni grafiche.

Esempio d’uso:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Con Il Thread Di Rendering

Individua i problemi del thread di rendering. Su Android moderno il disegno avviene su questo thread.

Esempio d’uso:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Con Il Rendering Della GPU

Individua i problemi di rendering della GPU. Si guarda qui dopo aver attivato il profilo di rendering GPU.

Esempio d’uso:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Con TextView

Individua i problemi delle viste di testo, come problemi di font ed errori di misura.

Esempio d’uso:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Con RecyclerView

Individua i problemi delle viste a elenco: prestazioni di scorrimento e riciclo delle viste.

Esempio d’uso:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Con ViewPager

Individua i problemi nei passaggi tra le pagine.

Esempio d’uso:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Con I Fragment

Individua i problemi del ciclo di vita dei fragment.

Esempio d’uso:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Con Le Activity

Individua i problemi del ciclo di vita delle activity.

Esempio d’uso:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Con I Servizi

Individua i problemi dei servizi in background.

Esempio d’uso:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Con WorkManager

Individua i problemi di WorkManager, per le attività in background.

Esempio d’uso:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Con AlarmManager

Individua i problemi di AlarmManager, per le attività pianificate.

Esempio d’uso:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Con SharedPreferences

Individua i problemi di SharedPreferences (archiviazione semplice di dati).

Esempio d’uso:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Con SQLite

Individua i problemi di SQLite (il database locale).

Esempio d’uso:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Con Le Operazioni Sui File

Individua i problemi di lettura e scrittura dei file.

Esempio d’uso:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Con Le Operazioni Di Rete

Individua i problemi di connessione di rete, come HTTP, TCP e UDP.

Esempio d’uso:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Con SSL

Individua i problemi dei certificati SSL. Si guarda qui quando le connessioni sicure falliscono.

Esempio d’uso:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Con JSON

Individua i problemi di analisi JSON.

Esempio d’uso:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Con XML

Individua i problemi di analisi XML.

Esempio d’uso:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Con Le Immagini

Individua i problemi di caricamento ed elaborazione delle immagini.

Esempio d’uso:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Con I Contenuti Multimediali

Individua i problemi di riproduzione e registrazione multimediale.

Esempio d’uso:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Con ExoPlayer

Individua i problemi di ExoPlayer (libreria di lettore multimediale).

Esempio d’uso:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Con Chromecast

Individua i problemi di Chromecast.

Esempio d’uso:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Con La Pubblicità

Individua i problemi delle librerie pubblicitarie.

Esempio d’uso:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Con La Fatturazione

Individua i problemi di fatturazione di Google Play.

Esempio d’uso:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Con Le Mappe

Individua i problemi delle librerie di mappe.

Esempio d’uso:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Con L’autenticazione

Individua i problemi di accesso, come Google Sign-In e Firebase Auth.

Esempio d’uso:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Con Le Notifiche

Individua i problemi di invio e ricezione delle notifiche.

Esempio d’uso:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Con I Deep Link

Individua i problemi dei deep link, come aprire un’app da un URL.

Esempio d’uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Con Gli Intent

Individua i problemi degli intent nel passaggio da un’app all’altra.

Esempio d’uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Con I Broadcast

Individua i problemi dei messaggi broadcast tra sistema e app.

Esempio d’uso:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Con I Content Provider

Individua i problemi dei content provider, per condividere dati tra app.

Esempio d’uso:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Con JobScheduler

Individua i problemi di JobScheduler, per le attività pianificate in background.

Esempio d’uso:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Con La Modalità Doze

Individua i problemi della modalità Doze (sospensione), per il lavoro in background mentre il dispositivo dorme.

Esempio d’uso:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Con L’ottimizzazione Della Batteria

Individua i problemi di ottimizzazione della batteria, per le restrizioni in background.

Esempio d’uso:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Con Le Perdite Di Memoria

Individua le perdite di memoria. Si usa insieme a strumenti come LeakCanary.

Esempio d’uso:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Con Gli ANR

Individua i problemi di ANR (l’app non risponde).

Esempio d’uso:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Con I Crash

Individua i crash.

Esempio d’uso:

```
adb logcat -b crash
```

---

135- Adb Logcat Con I Crash Nativi

Individua i crash nativi (C/C++/Rust).

Esempio d’uso:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Con L’analisi Dei Tombstone

Analizza i record dei crash nativi.

Esempio d’uso:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Con L’analisi Systrace

Analizza i record di tracciamento del sistema.

Esempio d’uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Con L’analisi Perfetto

Lo strumento moderno di tracciamento del sistema, più completo di logcat.

Esempio d’uso:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Con L’analisi Atrace

Per il tracciamento a livello di sistema; la forma più vecchia di Perfetto.

Esempio d’uso:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Con L’analisi Traceview

Per il profiling a livello di metodo.

Esempio d’uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Con Il Tracciamento Dei Metodi

Segue un metodo preciso.

Esempio d’uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Con Un Heap Dump

Crea un dump della memoria. Molto utile nell’analisi della memoria.

Esempio d’uso:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Con Il Log Del GC

Osserva gli eventi della garbage collection.

Esempio d’uso:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Con Il Debug JNI

Individua i problemi di JNI (Java Native Interface).

Esempio d’uso:

```
adb logcat -s JNI
```

---

145- Adb Logcat Con Il Debug ART

Individua i problemi di ART (Android Runtime).

Esempio d’uso:

```
adb logcat -s Art
```

---

146- Adb Logcat Con Il Debug Dalvik

Individua i problemi della macchina virtuale Dalvik. Si usa sulle versioni di Android più vecchie.

Esempio d’uso:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat E Il Problema Chatty

Quando logcat scrive troppi log, dà un avviso «chatty» e salta righe. Si risolve con un filtro o un buffer più grande.

Esempio d’uso (ingrandire il buffer):

```
adb logcat -G 16M
```

Esempio d’uso (usare un filtro):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Con Output A Colori

Nei flussi di log lunghi l’output a colori affatica meno gli occhi e fa vedere subito l’errore.

Esempio d’uso:

```
adb logcat -v color
```

Esempio d’uso (con time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Con Uno Script Bash

Serve a scrivere uno script che raccoglie i log in automatico. Su di esso si può costruire un sistema di segnalazione dei bug.

Script di esempio:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat In Sintesi

Come mostra questo elenco, logcat non è un singolo comando ma un insieme di comandi. Insieme a filtri, formati, buffer, file e altri strumenti diventa l’arma più potente del debug. Se sai quale combinazione usare in ogni situazione, trovare un bug richiede pochi minuti. Ricorda: la prima regola di una sessione di debug è sempre la stessa; prima svuota con adb logcat -c, poi riproduci il bug, poi osserva con un logcat filtrato.

@@ Connessione Android Debug Bridge
@: Comandi Di Connessione ADB Wireless
1- Attivare Il Debug Wireless

Prima bisogna attivare le opzioni sviluppatore sul dispositivo. Per farlo, tocca 7 volte Impostazioni → Informazioni sul telefono → Numero build. Poi attiva Impostazioni → Sistema → Opzioni sviluppatore → Debug wireless. Senza questo passaggio nessuno dei comandi adb pair e adb connect funziona.

Percorso di esempio:

```
Impostazioni → Informazioni sul telefono → Numero build (tocca 7 volte)
Impostazioni → Sistema → Opzioni sviluppatore → Debug wireless (attivo)
```

---

2- Trovare L’indirizzo IP E La Porta Del Dispositivo

Con il debug wireless attivo, apri l’opzione «Accoppia dispositivo con codice di accoppiamento». Lì compaiono l’indirizzo IP del dispositivo e un numero di porta casuale, ad esempio 192.168.1.42:37123. Questa porta serve solo per l’accoppiamento e poi cambia. Sulla stessa schermata c’è anche un codice di accoppiamento a 6 cifre.

Vista di esempio:

```
Indirizzo IP e porta
192.168.1.42:37123

Codice di accoppiamento Wi-Fi
482915
```

---

3- Il Comando Adb Pair

Questo comando accoppia il dispositivo. Si esegue su un computer o in Termux. Scrivi l’indirizzo IP e la porta di accoppiamento mostrati sul dispositivo. All’esecuzione il terminale chiede Enter pairing code: e attende il codice a 6 cifre. Se il codice è giusto compare il messaggio Successfully paired.

Esempio d’uso:

```
adb pair 192.168.1.42:37123
```

Esempio di output:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Con un codice sbagliato fallisce così:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Altri Modi Di Dare Il Codice Ad Adb Pair

Non è obbligatorio digitare il codice a mano ogni volta. Alcune versioni lo accettano direttamente come parametro, anche se per sicurezza di solito non è supportato. Si può comunque provare.

Esempio d’uso (in una riga):

```
adb pair 192.168.1.42:37123 482915
```

Esempio d’uso (con una pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Questi metodi cambiano a seconda della versione. Il modo più affidabile è eseguire il comando e digitare il codice a mano.

---

5- A Cosa Fare Attenzione Con Adb Pair

La porta di accoppiamento e quella di connessione sono diverse. La porta di accoppiamento di solito è casuale e alta, quella di connessione di solito è 5555. L’accoppiamento si fa una sola volta; per ricollegarsi allo stesso dispositivo basta adb connect. Ma se ripristini il dispositivo alle impostazioni di fabbrica o disattivi il debug wireless, l’accoppiamento si azzera e serve di nuovo adb pair.

Confronto di esempio:

```
Accoppiare: adb pair 192.168.1.42:37123   → chiede un codice, una volta sola
Collegare:  adb connect 192.168.1.42:5555  → nessun codice, ogni volta
```

---

6- Il Comando Adb Connect

Questo comando si collega a un dispositivo accoppiato. Scrivi l’indirizzo IP e la porta mostrati nella schermata principale del debug wireless. La porta di solito è 5555, ma a volte è diversa. Se la connessione riesce compare il messaggio Connected to.

Esempio d’uso:

```
adb connect 192.168.1.42:5555
```

Esempio di output:

```
Connected to 192.168.1.42:5555
```

Se è già collegato, avvisa così:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Senza Porta

In alcune versioni, se non si indica la porta, si prova 5555 per impostazione predefinita. Ma non avviene su tutti i dispositivi allo stesso modo, quindi scrivere la porta esplicitamente è la cosa più sicura.

Esempio d’uso (porta predefinita):

```
adb connect 192.168.1.42
```

Esempio d’uso (porta esplicita):

```
adb connect 192.168.1.42:5555
```

---

8- Verificare La Connessione

Dopo il collegamento, il comando adb devices mostra se il dispositivo è davvero collegato. L’indirizzo IP del dispositivo deve comparire nell’elenco con device accanto. Se c’è scritto offline, la connessione è stata stabilita ma non è ancora stabile; riprova poco dopo.

Esempio d’uso:

```
adb devices
```

Esempio di output:

```
List of devices attached
192.168.1.42:5555   device
```

Esempio di output (non collegato):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Il Comando Adb Disconnect

Chiude la connessione. La connessione adb termina anche se il dispositivo resta sulla stessa rete. Per ricollegarti esegui di nuovo adb connect; non serve accoppiare.

Esempio d’uso (un dispositivo):

```
adb disconnect 192.168.1.42:5555
```

Esempio di output:

```
Disconnected 192.168.1.42:5555
```

Esempio d’uso (tutti i dispositivi):

```
adb disconnect
```

Esempio di output:

```
Disconnected everything
```

---

10- Il Comando Adb Tcpip

Passa alla modalità wireless un dispositivo collegato via USB. Questo comando fa ascoltare il dispositivo sulla porta indicata, di solito 5555. È un metodo più diffuso nelle versioni precedenti ad Android 11, che non hanno adb pair.

Esempio d’uso:

```
adb tcpip 5555
```

Esempio di output:

```
Restarting in TCP mode port: 5555
```

Poi scollega il cavo USB ed esegui questo comando:

```
adb connect 192.168.1.42:5555
```

---

11- Il Comando Adb Usb

Torna dalla modalità wireless alla modalità USB, quando vuoi ricollegare il dispositivo con un cavo.

Esempio d’uso:

```
adb usb
```

Esempio di output:

```
Restarting in USB mode
```

---

12- Adb Kill-Server E Start-Server

A volte il server adb si blocca o rifiuta le connessioni. Allora bisogna fermarlo e riavviarlo. Questo azzera tutte le connessioni e dà un avvio pulito.

Esempio d’uso:

```
adb kill-server
adb start-server
```

Esempio di output:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Poi ricollegati:

```
adb connect 192.168.1.42:5555
```

---

13- Essere Sulla Stessa Rete

Per l’adb wireless il dispositivo e il computer o il telefono devono essere sulla stessa rete Wi-Fi. Su reti diverse non c’è connessione. Una rete ospiti, la divisione tra 5 GHz e 2,4 GHz, una VPN o un firewall a volte creano problemi. La soluzione semplice: collega entrambi i dispositivi allo stesso router e spegni le VPN.

Controllo di esempio:

```
Dispositivo: Impostazioni → Wi-Fi → Rete connessa (es. Home_Wifi)
Termux: Impostazioni → Wi-Fi → Rete connessa (es. Home_Wifi)
```

---

14- Trovare L’indirizzo IP A Mano

Per trovare l’indirizzo IP del dispositivo segui Impostazioni → Wi-Fi → Rete connessa → Dettagli rete. Oppure guardalo in Termux con il comando qui sotto. Ma l’indirizzo IP può cambiare nel tempo, quindi la cosa più affidabile è controllare ogni volta la schermata del debug wireless.

Esempio d’uso (Termux):

```
ifconfig wlan0
```

Esempio d’uso (sul dispositivo):

```
Impostazioni → Wi-Fi → Rete connessa → Indirizzo IP
```

---

15- Collegarsi A Sé Stesso Da Termux Sullo Stesso Dispositivo

Se Termux gira sullo stesso dispositivo, adb è già locale, ma ci si può comunque collegare tramite localhost con adb connect. Di solito si usa 127.0.0.1:5555. Alcuni dispositivi lo rifiutano; allora si usa l’IP locale del dispositivo stesso.

Esempio d’uso (localhost):

```
adb connect 127.0.0.1:5555
```

Esempio d’uso (il suo IP locale):

```
adb connect 192.168.1.42:5555
```

---

16- Eseguire Comandi Con Più Dispositivi Collegati

Se sono collegati più dispositivi insieme, scegli il dispositivo con il parametro -s in ogni comando. Altrimenti adb fallisce con error: more than one device.

Esempio d’uso:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Quando La Connessione Cade

Se il segnale Wi-Fi si indebolisce, il dispositivo va in sospensione o la rete cambia, la connessione cade. Il dispositivo allora sparisce dall’elenco di adb devices o compare offline. La soluzione: eseguire di nuovo adb connect. L’accoppiamento di solito resta valido e adb pair non serve.

Ripristino di esempio:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Dopo Il Riavvio Del Dispositivo

Al riavvio del dispositivo il debug wireless di solito si spegne. Su alcuni dispositivi resta acceso, ma l’accoppiamento si azzera. Quando il dispositivo si è avviato, vai nella schermata del debug wireless, prendi la nuova porta e il nuovo codice, esegui di nuovo adb pair e poi adb connect.

Flusso di esempio:

```
Dispositivo riavviato → attiva il debug wireless
Nuova porta: 42157, nuovo codice: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Errori Frequenti Durante L’accoppiamento

Failed: Wrong password → il codice è stato scritto male.
Failed to connect: Connection refused → la porta è sbagliata o il dispositivo non è in ascolto.
Failed: Pairing rejected → il dispositivo ha rifiutato l’accoppiamento; rinnova il codice.
Timeout → c’è un problema di rete; controlla di essere sulla stessa rete.

Errore di esempio e soluzione:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Soluzione: il codice sul dispositivo è cambiato; prendi il nuovo codice e riprova.

---

20- Spegnere L’adb Wireless

Spegnere il debug wireless quando hai finito è una buona idea per la sicurezza. Puoi spegnerlo dalle impostazioni o passare alla modalità USB con adb.

Esempio d’uso:

```
adb usb
adb disconnect
```

Esempio d’uso (dalle impostazioni):

```
Impostazioni → Sistema → Opzioni sviluppatore → Debug wireless (spento)
```

---

21- Differenze In Base Alla Versione Di Android

Android 11 e successivi: l’accoppiamento con adb pair è obbligatorio e si usa una porta casuale.
Android 10 e precedenti: si passa alla modalità wireless con adb tcpip 5555; non c’è accoppiamento.

Confronto di esempio:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Note Sulla Sicurezza

Mentre il debug wireless è attivo, qualcuno con cattive intenzioni sulla stessa rete potrebbe provare ad accedere al dispositivo. Per questo tieni la funzione spenta sulle reti Wi-Fi di cui non ti fidi. Il codice di accoppiamento esiste per la sicurezza, ma è comunque bene essere prudenti. Spegnilo quando hai finito.

Precauzione di esempio:

```
Quando non lo usi: Impostazioni → Opzioni sviluppatore → Debug wireless (spento)
Su una rete sicura: attivalo solo a casa o in ufficio
```

---

23- Tutto Il Flusso In Un Solo Posto

Lo script qui sotto mostra tutti i passaggi, da zero fino alla connessione. Basta cambiare IP e porta con quelli del tuo dispositivo.

Flusso di esempio:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Risultato atteso:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Uno Script Scorciatoia Per Una Connessione Duratura

Se ti colleghi spesso, puoi scrivere uno script e collegarti ogni volta con un solo comando. Ad esempio in Termux puoi aggiungere una funzione al file ~/.bashrc.

Funzione di esempio:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Inserisci IP:porta e il codice mostrati sul dispositivo:"
    read -p "Indirizzo: " ADRES
    adb pair "$ADRES"
    read -p "Porta di connessione (es. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Uso:

```
wifi_adb
```

---

25- Nota Finale

L’adb wireless permette il debug senza cavo USB ed è particolarmente comodo quando si lavora in Termux sul telefono stesso. I due comandi di base sono adb pair e adb connect; il primo accoppia una volta, il secondo si usa a ogni connessione. Gli altri (adb devices, adb disconnect, adb tcpip, adb kill-server) servono a sostenere questi due comandi e a risolvere i problemi. Non dimenticare la regola della stessa rete, spegnilo quando non lo usi e riavvia il server se qualcosa non va. Tutto qui.
"#;

const PORTUGUESE: &str = r#"@@ Comandos Do Logcat
1- Adb Logcat

Envia em direto para o terminal todos os registos do dispositivo. A partir do momento em que o comando corre, cada linha de registo produzida pelo sistema, pelas aplicações, pelos serviços e pelo kernel aparece no ecrã. Para parar, prima Ctrl + C no teclado. Este comando é a base da depuração; mas, sem filtro, pode fazer passar centenas de linhas por segundo e encontrar o que precisa torna-se difícil.

Exemplo de utilização:

```
adb logcat
```

Exemplo de saída:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Mostra os registos já acumulados e o comando termina. Em vez de um fluxo contínuo, tira uma fotografia desse momento. Utilize-o ao preparar um relatório de erros, ao guardar o estado após uma falha ou quando quer obter os registos uma única vez a partir de um script.

Exemplo de utilização:

```
adb logcat -d
```

Exemplo de utilização (guardar num ficheiro):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Este comando é muito usado em scripts que enviam relatórios automaticamente para um servidor, porque termina sozinho e nunca bloqueia o script.

---

3- Adb Logcat -C

Limpa todos os buffers de registo e apaga tudo o que foi acumulado até esse momento. Se o executar antes de uma nova sessão de depuração, só verá os registos produzidos a partir daí e o ruído antigo não o confundirá. Ao tentar reproduzir um erro, costuma ser o primeiro passo.

Exemplo de utilização:

```
adb logcat -c
adb logcat
```

Executando por ordem os dois comandos acima obtém-se um fluxo de registos totalmente limpo. Só aparece o que acontece depois da limpeza.

---

4- Adb Logcat -G

Mostra o tamanho dos buffers de registo atuais e o quanto estão cheios. Lista, linha a linha, a ocupação de cada buffer, a capacidade total e a quantidade usada. Se os registos passam muito depressa e as entradas antigas são descartadas (chama-se a isto «chatty»), veja aqui primeiro.

Exemplo de utilização:

```
adb logcat -g
```

Exemplo de saída:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Tamanho

Altera o tamanho do buffer de registo. Por exemplo, pode passar um buffer de 256 KiB para 16 MiB. Assim guardam-se mais registos e as entradas antigas são descartadas mais tarde. Muito útil em testes longos ou ao depurar uma aplicação muito ruidosa. A definição é reposta ao reiniciar o dispositivo.

Exemplo de utilização:

```
adb logcat -G 16M
```

Exemplo de utilização (para o reduzir):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Ficheiro

Escreve os registos no ficheiro indicado em vez de os mostrar no terminal. O terminal fica limpo e o registo é conservado. O caminho do ficheiro está no dispositivo, não num computador. Num terminal móvel (Termux) costuma escrever-se em /Sdcard/.

Exemplo de utilização:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Exemplo de utilização (acrescentando ao ficheiro):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

No exemplo acima abre-se um novo ficheiro quando o ficheiro chega a 4 MiB, e guardam-se 10 ficheiros no total.

---

7- Adb Logcat -B Main

Observa apenas o buffer principal das aplicações. Aqui ficam os registos que as aplicações escrevem com chamadas como Log.d, Log.i e Log.e. Já é observado por predefinição, mas indica-se explicitamente quando só se quer este buffer. É onde mais se olha ao desenvolver uma aplicação.

Exemplo de utilização:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Observa os registos dos serviços do sistema. Aqui ficam os registos de serviços como ActivityManager, WindowManager, PackageManager e PowerManager. Serve para ver eventos como o arranque de aplicações, a concessão de permissões e a ligação a serviços. Costuma observar-se em conjunto com main.

Exemplo de utilização:

```
adb logcat -b system
```

Exemplo de saída:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Observa apenas os registos de falhas. Aqui ficam os blocos FATAL EXCEPTION escritos quando uma aplicação é forçada a fechar. Ao depurar uma falha, olhar primeiro para este buffer é o caminho mais rápido, porque o rastreio da pilha está mesmo aqui.

Exemplo de utilização:

```
adb logcat -b crash
```

Exemplo de saída:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Observa os registos de rádio, telefonia, sinal e ligação de dados. Útil ao lidar com o cartão SIM, o registo na rede, a ligação 4G/5G, SMS ou chamadas. Normalmente não é preciso no desenvolvimento habitual de aplicações.

Exemplo de utilização:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Observa os registos binários dos eventos do sistema. Este buffer não contém texto normal, mas registos curtos codificados deixados pelas chamadas Log.event. Aparecem com etiquetas como am_proc_start e am_activity_launch. É usado na análise de desempenho.

Exemplo de utilização:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Observa os registos do kernel. Mostra as mensagens escritas pelo kernel Linux, que costumam parecer-se com a saída do comando dmesg. Aqui estão erros de controladores, problemas de memória e informação sobre temperatura e gestão de energia. Pode exigir root.

Exemplo de utilização:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Observa todos os buffers de uma vez. Main, system, crash, radio, events e kernel passam misturados. Útil quando se quer ver tudo, mas a saída fica muito cheia, por isso costuma combinar-se com um filtro.

Exemplo de utilização:

```
adb logcat -b all
```

Exemplo de utilização (com filtro):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Vários Buffers

Observa ao mesmo tempo vários buffers separados por vírgulas, por exemplo para ver juntos os registos da aplicação e das falhas. Escolhendo só os canais necessários, reduz a saída.

Exemplo de utilização:

```
adb logcat -b main,crash
```

Exemplo de utilização (três canais):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

O formato de saída curto predefinido. Cada linha começa apenas com a prioridade e a etiqueta. Bom para uma olhadela rápida, mas sem carimbo de hora é difícil ordenar os eventos.

Exemplo de utilização:

```
adb logcat -v brief
```

Exemplo de saída:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Acrescenta a data e a hora. Muito mais útil do que o formato curto, porque se vê quando cada evento aconteceu. Abaixo do segundo aparecem os milissegundos.

Exemplo de utilização:

```
adb logcat -v time
```

Exemplo de saída:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

O formato de saída mais recomendado. Inclui data, hora, prioridade, etiqueta, PID (identificador do processo) e TID (identificador da thread). Em aplicações com várias threads é indispensável para saber que thread fez o quê. É também o formato que o Android Studio usa por predefinição.

Exemplo de utilização:

```
adb logcat -v threadtime
```

Exemplo de saída:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Dá uma saída detalhada com todos os campos de metadados e uma linha em branco entre entradas. Em sessões longas mostra cada registo como um bloco, mas ocupa espaço.

Exemplo de utilização:

```
adb logcat -v long
```

Exemplo de saída:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colore os registos conforme a prioridade. Os erros ficam a vermelho, os avisos a laranja, a informação a verde e a depuração a azul. Num fluxo longo o olhar vai direto ao erro. O terminal tem de suportar cores.

Exemplo de utilização:

```
adb logcat -v color
```

Exemplo de utilização (em conjunto com time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Acrescenta o ano ao formato de hora. Usado em registos longos ou arquivos que abrangem mais de um ano. Normalmente não é preciso no desenvolvimento habitual.

Exemplo de utilização:

```
adb logcat -v year
```

Exemplo de saída:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Mostra o identificador de utilizador (UID) da aplicação que produziu o registo. Útil quando várias aplicações usam a mesma etiqueta. É um modificador, por isso combina-se com os outros formatos.

Exemplo de utilização:

```
adb logcat -v uid
```

Exemplo de saída:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filtro De Etiqueta E Prioridade

Mostra os registos de uma dada etiqueta a partir de uma dada prioridade. As prioridades são: V (Verbose) a mais baixa, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) e S (Silent) a mais alta. Esta forma de filtrar é a funcionalidade mais poderosa do logcat.

Exemplo de utilização (só erros):

```
adb logcat *:E
```

Exemplo de utilização (avisos e superiores):

```
adb logcat *:W
```

Exemplo de utilização (lista de permitidos):

```
adb logcat ActivityManager:I MyApp:D *:S
```

O último exemplo mostra ActivityManager a partir de info e MyApp a partir de debug, e silencia todas as outras etiquetas. Em algumas shells o carácter * é especial, por isso tem de ficar entre aspas:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Etiqueta

Um atalho do filtro silencioso. Mostra só a etiqueta indicada e silencia tudo o resto. A forma mais prática de se concentrar na sua própria etiqueta ao desenvolver.

Exemplo de utilização:

```
adb logcat -s MyApp
```

Exemplo de utilização (várias etiquetas):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

No exemplo acima só passam os registos de três etiquetas; tudo o resto fica em silêncio.

---

24- Adb Logcat -S Com Prioridade

Serve para indicar em conjunto a etiqueta e a prioridade, por exemplo para ver só os avisos e superiores da etiqueta MyApp.

Exemplo de utilização:

```
adb logcat MyApp:W *:S
```

Este comando mostra só os registos warning, error e fatal com a etiqueta MyApp.

---

25- Adb Logcat Filtro --Pid

Mostra só os registos de um dado identificador de processo (PID). Para saber o PID de uma aplicação, usa-se primeiro adb shell pidof e depois filtra-se por esse PID. Perfeito para isolar todos os registos de uma sessão.

Exemplo de utilização (obter o PID):

```
adb shell pidof com.example.app
```

Exemplo de saída:

```
5678
```

Exemplo de utilização (filtrar):

```
adb logcat --pid=5678
```

Exemplo de utilização (numa linha):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Hora

Mostra os registos a partir do carimbo de hora indicado, ou seja, «traz o que vem depois deste momento». Escreve-se com T maiúsculo; o t minúsculo significa outra coisa.

Exemplo de utilização:

```
adb logcat -T "09-21 14:00:00.000"
```

Exemplo de utilização (a partir de hoje):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Número De Linhas

Com t minúsculo mostra o número indicado de linhas mais recentes e o comando termina. Comporta-se como -d, mas dá só as últimas N linhas. Ideal para uma olhadela rápida.

Exemplo de utilização (últimas 100 linhas):

```
adb logcat -t 100
```

Exemplo de utilização (últimas 500 linhas e um filtro):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Procura no conteúdo com uma expressão regular (regex) e mostra as linhas que correspondem. Muito poderoso para pesquisas rápidas de texto; sobretudo ao procurar várias palavras de uma vez é mais rápido do que filtrar.

Exemplo de utilização (uma palavra):

```
adb logcat -e "Login"
```

Exemplo de utilização (várias palavras):

```
adb logcat -e "Login|Error|Crash"
```

Exemplo de utilização (com o nome do pacote):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Saída Binária

Escreve os registos em formato binário. As pessoas não o conseguem ler, mas outro logcat pode voltar a lê-lo ou outras ferramentas podem processá-lo. É usado em sistemas de análise automática.

Exemplo de utilização:

```
adb logcat -B > Ham_Log.Bin
```

Mais tarde esse ficheiro lê-se assim:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Estatísticas

Mostra estatísticas dos registos: quantos registos cada etiqueta produziu e que etiquetas dispararam o aviso «chatty». Se acha que uma aplicação está a inundar o registo, confirma-o com este comando.

Exemplo de utilização:

```
adb logcat -S
```

Exemplo de saída:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R E -N Em Conjunto

Para registos rotativos. -r define o tamanho de cada ficheiro e -n quantos ficheiros se guardam. Útil quando uma aplicação vai correr muito tempo e se quer registar continuamente. Os ficheiros antigos são apagados e abrem-se novos.

Exemplo de utilização:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

No exemplo acima abre-se um novo ficheiro sempre que um chega a 4 MiB e guardam-se 10 ficheiros no total. Assim o disco nunca enche sem fim.

---

32- Adb Logcat -P Script De Filtro

Indica um script de filtro personalizado para o logcat. É o formato de filtro próprio do Android e costuma ser usado por programadores do sistema. Os programadores de aplicações usam-no raramente.

Exemplo de utilização:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Carimbos De Hora

Uma opção que, em alguns formatos, acrescenta um carimbo de hora ao início de cada linha. O comportamento muda entre versões do Android. Costuma fazer o mesmo que -v time, mas de outra forma.

Exemplo de utilização:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Mostra cada linha em formato printf, ou seja, prioridade (PID) etiqueta: mensagem. Serve para compatibilidade com ferramentas antigas, mas raramente é preciso no Android moderno.

Exemplo de utilização:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Mostra só a mensagem em bruto, sem metadados. Escreve-se apenas o registo, sem etiqueta, prioridade nem carimbo de hora. Útil para passar a saída a outro programa.

Exemplo de utilização:

```
adb logcat -v raw
```

Exemplo de saída:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Escreve só a etiqueta no início de cada linha e depois o resto. A prioridade e o PID ficam ocultos. Prático quando se está concentrado numa etiqueta.

Exemplo de utilização:

```
adb logcat -v tag
```

Exemplo de saída:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Escreve primeiro o PID e depois a etiqueta no início de cada linha. Prático para análise por processo; vê-se depressa que processo escreveu o quê.

Exemplo de utilização:

```
adb logcat -v process
```

Exemplo de saída:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Escreve só o identificador da thread (TID) no início de cada linha e depois o resto. Em análises com várias threads mostra que thread está ativa. Um formato curto.

Exemplo de utilização:

```
adb logcat -v thread
```

Exemplo de saída:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D E -T Em Conjunto

Obtém os registos de um intervalo de tempo, por exemplo os da última hora, para os guardar num ficheiro.

Exemplo de utilização:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Exemplo de utilização (num ficheiro):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C E -D Em Sequência

Um fluxo que primeiro limpa e depois recolhe. Por exemplo, limpar antes de iniciar um teste e recolher os registos no fim do teste.

Exemplo de utilização:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Guardado Num Ficheiro E Levado Para O Computador

Prático para registar durante muito tempo num terminal móvel. Primeiro escreve-se num ficheiro no dispositivo e depois leva-se para o computador com adb pull.

Exemplo de utilização (guardar no dispositivo):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Exemplo de utilização (levar para o computador):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Com Grep

Usa-se num terminal móvel (Termux) para procurar texto depressa com grep. Pode ser mais flexível do que o filtro regex, porque tem todas as opções do grep.

Exemplo de utilização:

```
adb logcat | grep "Hata"
```

Exemplo de utilização (sem distinguir maiúsculas):

```
adb logcat | grep -I "error"
```

Exemplo de utilização (com números de linha):

```
adb logcat | grep -N "Exception"
```

Exemplo de utilização (várias palavras):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Com Tail

Reduz o fluxo em direto às suas últimas linhas, quando só quer ver o fim em vez de todo o registo.

Exemplo de utilização:

```
adb logcat | tail -100
```

Exemplo de utilização (seguir continuamente):

```
adb logcat | tail -F
```

---

44- Adb Logcat Com Head

Mostra as primeiras linhas. Bom para uma olhadela rápida a um registo longo.

Exemplo de utilização:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Com Wc

Mostra o número total de linhas, por exemplo para medir quantos registos um teste produziu.

Exemplo de utilização:

```
adb logcat -d | wc -l
```

Exemplo de utilização (contar só os erros):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Com Sort

Ordena os registos alfabética ou numericamente. É usado sobretudo durante a análise.

Exemplo de utilização:

```
adb logcat -d | sort
```

Exemplo de utilização (ordem inversa):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Com Uniq

Remove as linhas repetidas. Se uma aplicação escreve sempre o mesmo erro, este comando simplifica a saída.

Exemplo de utilização:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

O exemplo acima conta e mostra as 20 linhas mais repetidas. Muito útil na depuração.

---

48- Adb Logcat Com Awk

Extrai colunas específicas ou trabalha o texto, por exemplo a coluna PID no formato threadtime.

Exemplo de utilização (só a coluna PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Exemplo de utilização (só etiqueta e mensagem):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Com Xargs

Executa um comando por cada linha, por exemplo enviar uma notificação por cada linha de erro. É usado sobretudo em automação.

Exemplo de utilização:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Com Sed

Substitui texto. Ideal para alterar padrões específicos na saída.

Exemplo de utilização:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

O exemplo acima escreve Uygulama no lugar de cada MyApp.

---

51- Adb Logcat Com Tee

Mostra no ecrã e escreve num ficheiro ao mesmo tempo. A observação em direto e o registo acontecem juntos.

Exemplo de utilização:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Com Watch

Obtém os registos em intervalos fixos, por exemplo um resumo a cada 5 segundos.

Exemplo de utilização:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Com Timeout

Para o logcat ao fim de um tempo definido. Muito útil em testes automáticos.

Exemplo de utilização:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Com Alias

Encurta os comandos que usa com frequência. No Termux adiciona-se ao ficheiro .bashrc ou .zshrc.

Exemplo de utilização (definir um alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Depois basta chamá-lo assim:

```
lce
```

---

55- Adb Logcat Com Vários Dispositivos

Aponta para um dispositivo específico quando há vários ligados. Primeiro listam-se os dispositivos com adb devices e depois escolhe-se um com -s.

Exemplo de utilização:

```
adb devices
```

Exemplo de saída:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Exemplo de utilização (um dispositivo específico):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Com Depuração Sem Fios

Para a depuração sem fios. Quando o dispositivo está na mesma rede Wi-Fi, a ligação faz-se sem cabo. Suportado a partir do Android 11.

Exemplo de utilização (ative a depuração sem fios no dispositivo e depois):

```
adb pair 192.168.1.42:37123
```

Exemplo de utilização (ligação):

```
adb connect 192.168.1.42:5555
```

Exemplo de utilização (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Com Bugreport

Cria um relatório detalhado de todo o sistema. Além do logcat são executadas muitas ferramentas como dumpsys, dmesg e top, e tudo fica num único zip. É a forma de depuração mais completa.

Exemplo de utilização:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Exemplo de utilização (levar para o computador):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Com Dumpsys

Mostra, ao lado do logcat, o estado atual dos serviços do sistema, por exemplo o que o ActivityManager está a fazer nesse momento.

Exemplo de utilização:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Com Rastreios De ANR

Obtém os rastreios de pilha quando uma aplicação não responde (ANR). Os registos de ANR ficam no ficheiro /Data/Anr/Traces.Txt.

Exemplo de utilização:

```
adb shell cat /Data/Anr/Traces.Txt
```

Exemplo de utilização (requer root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Com Tombstones

Para falhas nativas. Na pasta /Data/Tombstones/ é criado um ficheiro por cada falha. É aqui que se procura quando o código C++ ou Rust tem um erro.

Exemplo de utilização:

```
adb shell ls /Data/Tombstones/
```

Exemplo de utilização (o último tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Com Systrace

Para o rastreio de desempenho de todo o sistema. Recolhe dados de rastreio ao nível do kernel, não apenas registos. É usado em conjunto com o Perfetto.

Exemplo de utilização:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Com Layout Inspector

Analisa o layout de uma aplicação no ecrã. É usado com o Layout Inspector, mas os registos relacionados também podem ser seguidos com o logcat.

Exemplo de utilização:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Com Memory Profiler

Observa o uso de memória. O Memory Profiler costuma estar no Android Studio, mas os registos ligados à memória também se veem no logcat.

Exemplo de utilização:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Com CPU Profiler

Observa o uso do CPU. Para uma medição simples também serve o comando top.

Exemplo de utilização:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Com Network Profiler

Observa o tráfego de rede. Bibliotecas como OkHttp e Retrofit escrevem no logcat, mas uma análise completa precisa do Network Profiler.

Exemplo de utilização:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Com GPU Profiler

Observa o uso da GPU. Para uma medição simples também serve o comando gfxinfo.

Exemplo de utilização:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Com Battery Historian

Observa o consumo da bateria. Também se pode ver com dumpsys battery.

Exemplo de utilização:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Para Diagnosticar O Wi-Fi

Serve para depurar problemas de Wi-Fi, como quedas de ligação ou sinal fraco.

Exemplo de utilização:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Para Diagnosticar O Bluetooth

Serve para depurar problemas de Bluetooth, como falhas de emparelhamento ou desconexões.

Exemplo de utilização:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Para Diagnosticar Os Sensores

Serve para depurar problemas de sensores como o acelerómetro, o giroscópio e o magnetómetro.

Exemplo de utilização:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Para Diagnosticar A Câmara

Serve para depurar problemas da câmara, como tirar fotografias ou gravar vídeo.

Exemplo de utilização:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Para Diagnosticar O Áudio

Serve para depurar problemas de áudio, como falta de som ou um microfone que não funciona.

Exemplo de utilização:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Para Diagnosticar A Localização

Serve para depurar problemas de localização, como falta de sinal GPS ou uma localização que não se atualiza.

Exemplo de utilização:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Para Diagnosticar As Notificações

Serve para depurar problemas de notificações, como notificações que não aparecem ou chegam na altura errada.

Exemplo de utilização:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Para Diagnosticar As Permissões

Serve para depurar problemas de permissões, como uma permissão não concedida ou recusada.

Exemplo de utilização:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Para Diagnosticar O WebView

Serve para depurar problemas do WebView, como uma página que não carrega ou um erro de JavaScript.

Exemplo de utilização:

```
adb logcat -s Chromium WebView
```

Exemplo de utilização (mais detalhes):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Para Diagnosticar O OkHttp

Serve para depurar problemas do OkHttp: pedidos de rede, respostas e erros.

Exemplo de utilização:

```
adb logcat -s Okhttp
```

Exemplo de utilização (todos os níveis):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Para Diagnosticar O Retrofit

Serve para depurar problemas do Retrofit. Como costuma assentar no OkHttp, olha-se para os registos deste.

Exemplo de utilização:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Para Diagnosticar O Glide

Serve para depurar problemas do Glide (biblioteca de carregamento de imagens), como imagens que não carregam ou problemas de memória.

Exemplo de utilização:

```
adb logcat -s Glide
```

---

80- Adb Logcat Para Diagnosticar O Picasso

Serve para depurar problemas do Picasso (biblioteca de carregamento de imagens).

Exemplo de utilização:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Para Diagnosticar O Room

Serve para depurar problemas do Room (biblioteca de base de dados): erros de consulta e problemas de esquema.

Exemplo de utilização:

```
adb logcat -s Room
```

---

82- Adb Logcat Para Diagnosticar O Firebase

Serve para depurar problemas do Firebase, em módulos como Crashlytics, Analytics e Messaging.

Exemplo de utilização:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Para Diagnosticar O Crashlytics

Serve para depurar problemas do Firebase Crashlytics.

Exemplo de utilização:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Para Diagnosticar O Analytics

Serve para depurar problemas de análise: eventos não enviados e problemas de envio em lote.

Exemplo de utilização:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Com Depuração ADB Sem Fios

A depuração sem fios às vezes exige reiniciar o adb. Ligue-se primeiro por USB e depois passe para sem fios.

Exemplo de utilização:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Durante O Reinício Do Dispositivo

Reinicia o dispositivo e observa os registos, sobretudo para erros que acontecem no arranque.

Exemplo de utilização (dois terminais):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Durante O Reinício De Uma Aplicação

Reinicia uma única aplicação e observa os registos.

Exemplo de utilização:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Durante A Instalação De Uma Aplicação

Instala um APK e observa os registos.

Exemplo de utilização:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Durante A Desinstalação De Uma Aplicação

Desinstala uma aplicação e observa os registos.

Exemplo de utilização:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Com Captura De Ecrã

Tira uma captura de ecrã e guarda o registo no momento de um erro.

Exemplo de utilização:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Com Gravação Do Ecrã

Grava o ecrã e guarda o registo no momento de um erro. Suportado a partir do Android 4.4.

Exemplo de utilização:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Com Teste Monkey

Testa uma aplicação simulando interações aleatórias do utilizador. Muito eficaz em conjunto com os registos.

Exemplo de utilização:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Com Teste UI Automator

Para testes de automação da interface, observando os registos durante o teste.

Exemplo de utilização:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Com Teste Espresso

Para testes Espresso, um subconjunto da automação da interface.

Exemplo de utilização:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Com Teste JUnit

Para testes unitários. Normalmente correm num computador, mas também podem correr no dispositivo.

Exemplo de utilização:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Com StrictMode

Deteta problemas de desempenho de uma aplicação. Avisa quando se acede ao disco ou à rede na thread principal.

Exemplo de utilização:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Com Choreographer

Deteta fotogramas saltados. É aqui que se procura quando as animações engasgam.

Exemplo de utilização:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Com Overdraw Da GPU

Deteta problemas de overdraw. É usado na análise do desempenho gráfico.

Exemplo de utilização:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Com A Thread De Renderização

Deteta problemas da thread de renderização. No Android moderno o desenho acontece nesta thread.

Exemplo de utilização:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Com Renderização Da GPU

Deteta problemas de renderização da GPU. É aqui que se procura depois de ativar o perfil de renderização da GPU.

Exemplo de utilização:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Com TextView

Deteta problemas das vistas de texto, como problemas de tipo de letra e erros de medição.

Exemplo de utilização:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Com RecyclerView

Deteta problemas das vistas de lista: desempenho do deslocamento e reciclagem das vistas.

Exemplo de utilização:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Com ViewPager

Deteta problemas nas transições entre páginas.

Exemplo de utilização:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Com Fragments

Deteta problemas do ciclo de vida dos fragments.

Exemplo de utilização:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Com Activities

Deteta problemas do ciclo de vida das activities.

Exemplo de utilização:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Com Serviços

Deteta problemas dos serviços em segundo plano.

Exemplo de utilização:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Com WorkManager

Deteta problemas do WorkManager, para tarefas em segundo plano.

Exemplo de utilização:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Com AlarmManager

Deteta problemas do AlarmManager, para tarefas agendadas.

Exemplo de utilização:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Com SharedPreferences

Deteta problemas de SharedPreferences (armazenamento simples de dados).

Exemplo de utilização:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Com SQLite

Deteta problemas de SQLite (a base de dados local).

Exemplo de utilização:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Com Operações De Ficheiros

Deteta problemas de leitura e escrita de ficheiros.

Exemplo de utilização:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Com Operações De Rede

Deteta problemas de ligação de rede, como HTTP, TCP e UDP.

Exemplo de utilização:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Com SSL

Deteta problemas de certificados SSL. É aqui que se procura quando as ligações seguras falham.

Exemplo de utilização:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Com JSON

Deteta problemas de análise de JSON.

Exemplo de utilização:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Com XML

Deteta problemas de análise de XML.

Exemplo de utilização:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Com Imagens

Deteta problemas de carregamento e processamento de imagens.

Exemplo de utilização:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Com Multimédia

Deteta problemas de reprodução e gravação multimédia.

Exemplo de utilização:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Com ExoPlayer

Deteta problemas do ExoPlayer (biblioteca de leitor multimédia).

Exemplo de utilização:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Com Chromecast

Deteta problemas do Chromecast.

Exemplo de utilização:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Com Publicidade

Deteta problemas das bibliotecas de publicidade.

Exemplo de utilização:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Com Faturação

Deteta problemas de faturação do Google Play.

Exemplo de utilização:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Com Mapas

Deteta problemas das bibliotecas de mapas.

Exemplo de utilização:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Com Autenticação

Deteta problemas de início de sessão, como o Google Sign-In e o Firebase Auth.

Exemplo de utilização:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Com Notificações

Deteta problemas de envio e receção de notificações.

Exemplo de utilização:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Com Deep Links

Deteta problemas de deep links, como abrir uma aplicação a partir de um URL.

Exemplo de utilização:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Com Intents

Deteta problemas dos intents na passagem entre aplicações.

Exemplo de utilização:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Com Broadcasts

Deteta problemas das mensagens broadcast entre o sistema e as aplicações.

Exemplo de utilização:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Com Content Providers

Deteta problemas dos content providers, para partilhar dados entre aplicações.

Exemplo de utilização:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Com JobScheduler

Deteta problemas do JobScheduler, para tarefas agendadas em segundo plano.

Exemplo de utilização:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Com O Modo Doze

Deteta problemas do modo Doze (suspensão), para o trabalho em segundo plano enquanto o dispositivo dorme.

Exemplo de utilização:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Com Otimização Da Bateria

Deteta problemas de otimização da bateria, para as restrições em segundo plano.

Exemplo de utilização:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Com Fugas De Memória

Deteta fugas de memória. É usado em conjunto com ferramentas como o LeakCanary.

Exemplo de utilização:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Com ANR

Deteta problemas de ANR (a aplicação não responde).

Exemplo de utilização:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Com Falhas

Deteta falhas.

Exemplo de utilização:

```
adb logcat -b crash
```

---

135- Adb Logcat Com Falhas Nativas

Deteta falhas nativas (C/C++/Rust).

Exemplo de utilização:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Com Análise De Tombstones

Analisa os registos das falhas nativas.

Exemplo de utilização:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Com Análise Systrace

Analisa os registos de rastreio do sistema.

Exemplo de utilização:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Com Análise Perfetto

A ferramenta moderna de rastreio do sistema, mais completa do que o logcat.

Exemplo de utilização:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Com Análise Atrace

Para o rastreio ao nível do sistema; a forma mais antiga do Perfetto.

Exemplo de utilização:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Com Análise Traceview

Para o profiling ao nível dos métodos.

Exemplo de utilização:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Com Rastreio De Métodos

Segue um método específico.

Exemplo de utilização:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Com Heap Dump

Cria um despejo da memória. Muito útil na análise da memória.

Exemplo de utilização:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Com Registo Do GC

Observa os eventos da recolha de lixo.

Exemplo de utilização:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Com Depuração JNI

Deteta problemas de JNI (Java Native Interface).

Exemplo de utilização:

```
adb logcat -s JNI
```

---

145- Adb Logcat Com Depuração ART

Deteta problemas do ART (Android Runtime).

Exemplo de utilização:

```
adb logcat -s Art
```

---

146- Adb Logcat Com Depuração Dalvik

Deteta problemas da máquina virtual Dalvik. É usado nas versões mais antigas do Android.

Exemplo de utilização:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat E O Problema Chatty

Quando o logcat escreve registos a mais, dá um aviso «chatty» e salta linhas. Resolve-se com um filtro ou um buffer maior.

Exemplo de utilização (aumentar o buffer):

```
adb logcat -G 16M
```

Exemplo de utilização (usar um filtro):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Com Saída A Cores

Em fluxos de registos longos, a saída a cores cansa menos os olhos e faz ver o erro de imediato.

Exemplo de utilização:

```
adb logcat -v color
```

Exemplo de utilização (com time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Com Um Script Bash

Serve para escrever um script que recolhe os registos automaticamente. Sobre ele pode construir-se um sistema de relatório de erros.

Script de exemplo:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat Em Resumo

Como esta lista mostra, o logcat não é um único comando mas um conjunto de comandos. Junto com filtros, formatos, buffers, ficheiros e outras ferramentas, torna-se a arma mais poderosa da depuração. Se souber que combinação usar em cada situação, encontrar um erro leva poucos minutos. Lembre-se: a primeira regra de uma sessão de depuração é sempre a mesma; primeiro limpe com adb logcat -c, depois reproduza o erro e depois observe com um logcat filtrado.

@@ Ligação Do Android Debug Bridge
@: Comandos De Ligação ADB Sem Fios
1- Ativar A Depuração Sem Fios

Primeiro é preciso ativar as opções de programador no dispositivo. Para isso, toque 7 vezes em Definições → Acerca do telefone → Número de compilação. Depois ative Definições → Sistema → Opções de programador → Depuração sem fios. Sem este passo nenhum dos comandos adb pair e adb connect funciona.

Caminho de exemplo:

```
Definições → Acerca do telefone → Número de compilação (tocar 7 vezes)
Definições → Sistema → Opções de programador → Depuração sem fios (ativada)
```

---

2- Encontrar O Endereço IP E A Porta Do Dispositivo

Com a depuração sem fios ativada, abra a opção «Sincronizar dispositivo com código de sincronização». Aí aparecem o endereço IP do dispositivo e um número de porta aleatório, por exemplo 192.168.1.42:37123. Esta porta serve apenas para o emparelhamento e depois muda. No mesmo ecrã há também um código de emparelhamento de 6 dígitos.

Vista de exemplo:

```
Endereço IP e porta
192.168.1.42:37123

Código de sincronização Wi-Fi
482915
```

---

3- O Comando Adb Pair

Este comando emparelha o dispositivo. É executado num computador ou no Termux. Escreva o endereço IP e a porta de emparelhamento mostrados no dispositivo. Ao ser executado, o terminal pergunta Enter pairing code: e espera pelo código de 6 dígitos. Se o código estiver certo, aparece a mensagem Successfully paired.

Exemplo de utilização:

```
adb pair 192.168.1.42:37123
```

Exemplo de saída:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Com um código errado falha assim:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Outras Formas De Dar O Código Ao Adb Pair

Não é obrigatório escrever o código à mão todas as vezes. Algumas versões aceitam-no diretamente como parâmetro, embora por segurança isso normalmente não seja suportado. Mesmo assim pode experimentar.

Exemplo de utilização (numa linha):

```
adb pair 192.168.1.42:37123 482915
```

Exemplo de utilização (com um pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Estes métodos mudam consoante a versão. A forma mais fiável é executar o comando e escrever o código à mão.

---

5- Cuidados A Ter Com O Adb Pair

A porta de emparelhamento e a porta de ligação são diferentes. A porta de emparelhamento costuma ser aleatória e alta; a de ligação costuma ser 5555. O emparelhamento faz-se uma única vez; para voltar a ligar ao mesmo dispositivo basta o adb connect. Mas se repuser o dispositivo de fábrica ou desativar a depuração sem fios, o emparelhamento é apagado e é preciso fazer adb pair outra vez.

Comparação de exemplo:

```
Emparelhar: adb pair 192.168.1.42:37123   → pede um código, só uma vez
Ligar:      adb connect 192.168.1.42:5555  → sem código, todas as vezes
```

---

6- O Comando Adb Connect

Este comando liga a um dispositivo emparelhado. Escreva o endereço IP e a porta mostrados no ecrã principal da depuração sem fios. A porta costuma ser 5555, mas às vezes é diferente. Se a ligação for bem-sucedida aparece a mensagem Connected to.

Exemplo de utilização:

```
adb connect 192.168.1.42:5555
```

Exemplo de saída:

```
Connected to 192.168.1.42:5555
```

Se já estiver ligado, avisa assim:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Sem Porta

Em algumas versões, se não indicar a porta, é tentada a 5555 por omissão. Mas isso não acontece da mesma forma em todos os dispositivos, por isso escrever a porta explicitamente é o mais seguro.

Exemplo de utilização (porta por omissão):

```
adb connect 192.168.1.42
```

Exemplo de utilização (porta explícita):

```
adb connect 192.168.1.42:5555
```

---

8- Verificar A Ligação

Depois de ligar, o comando adb devices mostra se o dispositivo está mesmo ligado. O endereço IP do dispositivo tem de aparecer na lista com device ao lado. Se disser offline, a ligação foi estabelecida mas ainda não está estável; tente outra vez daqui a pouco.

Exemplo de utilização:

```
adb devices
```

Exemplo de saída:

```
List of devices attached
192.168.1.42:5555   device
```

Exemplo de saída (não ligado):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- O Comando Adb Disconnect

Termina a ligação. A ligação adb termina mesmo que o dispositivo continue na mesma rede. Para voltar a ligar execute outra vez o adb connect; não é preciso emparelhar.

Exemplo de utilização (um dispositivo):

```
adb disconnect 192.168.1.42:5555
```

Exemplo de saída:

```
Disconnected 192.168.1.42:5555
```

Exemplo de utilização (todos os dispositivos):

```
adb disconnect
```

Exemplo de saída:

```
Disconnected everything
```

---

10- O Comando Adb Tcpip

Passa para o modo sem fios um dispositivo ligado por USB. Este comando põe o dispositivo à escuta na porta indicada, normalmente a 5555. É um método mais comum nas versões anteriores ao Android 11, que não têm adb pair.

Exemplo de utilização:

```
adb tcpip 5555
```

Exemplo de saída:

```
Restarting in TCP mode port: 5555
```

Depois desligue o cabo USB e execute este comando:

```
adb connect 192.168.1.42:5555
```

---

11- O Comando Adb Usb

Volta do modo sem fios para o modo USB, quando quer ligar outra vez o dispositivo com um cabo.

Exemplo de utilização:

```
adb usb
```

Exemplo de saída:

```
Restarting in USB mode
```

---

12- Adb Kill-Server E Start-Server

Às vezes o servidor adb bloqueia ou recusa ligações. Nessa altura é preciso pará-lo e voltar a iniciá-lo. Isto repõe todas as ligações e dá um arranque limpo.

Exemplo de utilização:

```
adb kill-server
adb start-server
```

Exemplo de saída:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Depois volte a ligar:

```
adb connect 192.168.1.42:5555
```

---

13- Estar Na Mesma Rede

Para o adb sem fios, o dispositivo e o computador ou telemóvel têm de estar na mesma rede Wi-Fi. Em redes diferentes não há ligação. Uma rede de convidados, a divisão entre 5 GHz e 2,4 GHz, uma VPN ou uma firewall às vezes causam problemas. A solução simples: ligue os dois dispositivos ao mesmo router e desligue as VPN.

Verificação de exemplo:

```
Dispositivo: Definições → Wi-Fi → Rede ligada (ex. Home_Wifi)
Termux: Definições → Wi-Fi → Rede ligada (ex. Home_Wifi)
```

---

14- Encontrar O Endereço IP À Mão

Para encontrar o endereço IP do dispositivo siga Definições → Wi-Fi → Rede ligada → Detalhes da rede. Ou veja-o no Termux com o comando abaixo. Mas o endereço IP pode mudar com o tempo, por isso o mais fiável é verificar sempre o ecrã da depuração sem fios.

Exemplo de utilização (Termux):

```
ifconfig wlan0
```

Exemplo de utilização (no dispositivo):

```
Definições → Wi-Fi → Rede ligada → Endereço IP
```

---

15- Ligar-Se A Si Próprio A Partir Do Termux No Mesmo Dispositivo

Se o Termux corre no mesmo dispositivo, o adb já é local, mas mesmo assim pode ligar-se através de localhost com adb connect. Normalmente usa-se 127.0.0.1:5555. Alguns dispositivos recusam; nesse caso usa-se o IP local do próprio dispositivo.

Exemplo de utilização (localhost):

```
adb connect 127.0.0.1:5555
```

Exemplo de utilização (o seu IP local):

```
adb connect 192.168.1.42:5555
```

---

16- Executar Comandos Com Vários Dispositivos Ligados

Se estiverem vários dispositivos ligados ao mesmo tempo, escolha o dispositivo com o parâmetro -s em cada comando. Caso contrário o adb falha com error: more than one device.

Exemplo de utilização:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Quando A Ligação Cai

Se o sinal Wi-Fi enfraquecer, o dispositivo entrar em suspensão ou a rede mudar, a ligação cai. O dispositivo desaparece então da lista do adb devices ou aparece offline. A solução: executar outra vez o adb connect. O emparelhamento normalmente continua válido e o adb pair não é necessário.

Recuperação de exemplo:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Depois De Reiniciar O Dispositivo

Quando o dispositivo reinicia, a depuração sem fios normalmente desliga-se. Em alguns dispositivos continua ligada, mas o emparelhamento é apagado. Quando o dispositivo arrancar, vá ao ecrã da depuração sem fios, obtenha a nova porta e o novo código, execute outra vez o adb pair e depois o adb connect.

Fluxo de exemplo:

```
Dispositivo reiniciado → ativar a depuração sem fios
Nova porta: 42157, novo código: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Erros Frequentes No Emparelhamento

Failed: Wrong password → o código foi mal escrito.
Failed to connect: Connection refused → a porta está errada ou o dispositivo não está à escuta.
Failed: Pairing rejected → o dispositivo recusou o emparelhamento; renove o código.
Timeout → há um problema de rede; verifique se estão na mesma rede.

Erro de exemplo e solução:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Solução: o código no dispositivo mudou; obtenha o novo código e tente outra vez.

---

20- Desligar O Adb Sem Fios

Desligar a depuração sem fios quando terminar é uma boa ideia para a segurança. Pode desligá-la nas definições ou passar para o modo USB com o adb.

Exemplo de utilização:

```
adb usb
adb disconnect
```

Exemplo de utilização (nas definições):

```
Definições → Sistema → Opções de programador → Depuração sem fios (desativada)
```

---

21- Diferenças Consoante A Versão Do Android

Android 11 e posteriores: o emparelhamento com adb pair é obrigatório e usa-se uma porta aleatória.
Android 10 e anteriores: passa-se para o modo sem fios com adb tcpip 5555; não há emparelhamento.

Comparação de exemplo:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Notas De Segurança

Enquanto a depuração sem fios está ativa, alguém mal-intencionado na mesma rede pode tentar aceder ao dispositivo. Por isso mantenha a funcionalidade desligada em redes Wi-Fi em que não confia. O código de emparelhamento existe por segurança, mas mesmo assim convém ter cuidado. Desligue-a quando terminar.

Precaução de exemplo:

```
Quando não estiver a usar: Definições → Opções de programador → Depuração sem fios (desativada)
Numa rede segura: ative-a só em casa ou no escritório
```

---

23- Todo O Fluxo Num Só Lugar

O script abaixo mostra todos os passos, do zero até à ligação. Basta trocar o IP e a porta pelos do seu dispositivo.

Fluxo de exemplo:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Resultado esperado:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Um Script De Atalho Para Uma Ligação Duradoura

Se se liga com frequência, pode escrever um script e ligar-se sempre com um único comando. Por exemplo, no Termux pode adicionar uma função ao ficheiro ~/.bashrc.

Função de exemplo:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Introduza o IP:porta e o código mostrados no dispositivo:"
    read -p "Endereço: " ADRES
    adb pair "$ADRES"
    read -p "Porta de ligação (ex. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Utilização:

```
wifi_adb
```

---

25- Nota Final

O adb sem fios permite depurar sem cabo USB e é especialmente prático quando se trabalha no Termux no próprio telemóvel. Os dois comandos básicos são adb pair e adb connect; o primeiro emparelha uma vez, o segundo usa-se em cada ligação. Os restantes (adb devices, adb disconnect, adb tcpip, adb kill-server) servem para apoiar estes dois comandos e resolver problemas. Não se esqueça da regra da mesma rede, desligue-o quando não o usar e reinicie o servidor se algo correr mal. É tudo.
"#;

const RUSSIAN: &str = r#"@@ Команды Logcat
1- Adb Logcat

Выводит в терминал в реальном времени все журналы устройства. С момента запуска команды на экране появляется каждая строка журнала, которую создают система, приложения, службы и ядро. Чтобы остановить вывод, нажмите Ctrl + C на клавиатуре. Эта команда — основа отладки; но без фильтра она может выдавать сотни строк в секунду, и найти нужное становится трудно.

Пример использования:

```
adb logcat
```

Пример вывода:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Показывает уже накопленные журналы, и команда завершается. Вместо непрерывного потока она делает снимок этого момента. Используйте её при подготовке отчёта об ошибке, для сохранения состояния после сбоя или когда нужно один раз получить журналы из скрипта.

Пример использования:

```
adb logcat -d
```

Пример использования (сохранение в файл):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Эта команда часто применяется в скриптах, которые автоматически отправляют отчёты на сервер, потому что она завершается сама и никогда не блокирует скрипт.

---

3- Adb Logcat -C

Очищает все буферы журналов и удаляет всё, что накопилось к этому моменту. Если выполнить её перед новым сеансом отладки, вы увидите только журналы, появившиеся после этого, и старый шум не будет мешать. При попытке воспроизвести ошибку это обычно первый шаг.

Пример использования:

```
adb logcat -c
adb logcat
```

Если выполнить две команды выше по порядку, получится полностью чистый поток журналов. Появится только то, что происходит после очистки.

---

4- Adb Logcat -G

Показывает размер текущих буферов журналов и насколько они заполнены. Построчно перечисляет занятость каждого буфера, общую ёмкость и использованный объём. Если журналы идут слишком быстро и старые записи отбрасываются (это называется «chatty»), сначала посмотрите сюда.

Пример использования:

```
adb logcat -g
```

Пример вывода:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Размер

Изменяет размер буфера журналов. Например, буфер размером 256 KiB можно увеличить до 16 MiB. Так хранится больше журналов, и старые записи отбрасываются позже. Очень полезно в долгих тестах или при отладке очень «шумного» приложения. Настройка сбрасывается при перезагрузке устройства.

Пример использования:

```
adb logcat -G 16M
```

Пример использования (уменьшить):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Файл

Записывает журналы в указанный файл вместо вывода в терминал. Терминал остаётся чистым, а журнал сохраняется. Путь к файлу находится на устройстве, а не на компьютере. В мобильном терминале (Termux) обычно пишут в /Sdcard/.

Пример использования:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Пример использования (с дописыванием в файл):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

В примере выше, когда файл достигает 4 MiB, открывается новый файл, и всего хранится 10 файлов.

---

7- Adb Logcat -B Main

Следит только за основным буфером приложений. Сюда попадают журналы, которые приложения пишут вызовами Log.d, Log.i и Log.e. Он и так отслеживается по умолчанию, но его указывают явно, когда нужен только этот буфер. Именно сюда чаще всего смотрят при разработке приложения.

Пример использования:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Следит за журналами системных служб. Сюда попадают журналы таких служб, как ActivityManager, WindowManager, PackageManager и PowerManager. Позволяет видеть такие события, как запуск приложений, выдача разрешений и подключение к службам. Обычно просматривается вместе с main.

Пример использования:

```
adb logcat -b system
```

Пример вывода:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Следит только за журналами сбоев. Сюда попадают блоки FATAL EXCEPTION, которые пишутся при принудительном закрытии приложения. При отладке сбоя сначала смотреть в этот буфер — самый быстрый путь, потому что трассировка стека находится прямо здесь.

Пример использования:

```
adb logcat -b crash
```

Пример вывода:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Следит за журналами радиомодуля, телефонии, сигнала и передачи данных. Полезно при работе с SIM-картой, регистрацией в сети, подключением 4G/5G, SMS или звонками. В обычной разработке приложений, как правило, не нужен.

Пример использования:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Следит за двоичными журналами системных событий. В этом буфере не обычный текст, а короткие закодированные записи, оставленные вызовами Log.event. Они появляются с тегами вроде am_proc_start и am_activity_launch. Используется при анализе производительности.

Пример использования:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Следит за журналами ядра. Показывает сообщения, которые пишет ядро Linux; обычно они похожи на вывод команды dmesg. Здесь есть ошибки драйверов, проблемы с памятью, сведения о температуре и управлении питанием. Может потребоваться root.

Пример использования:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Следит сразу за всеми буферами. Main, system, crash, radio, events и kernel идут вперемешку. Полезно, когда нужно видеть всё, но вывод получается очень плотным, поэтому его обычно сочетают с фильтром.

Пример использования:

```
adb logcat -b all
```

Пример использования (с фильтром):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Несколько Буферов

Следит одновременно за несколькими буферами, перечисленными через запятую, например чтобы видеть вместе журналы приложения и сбоев. Выбирая только нужные каналы, вы сокращаете вывод.

Пример использования:

```
adb logcat -b main,crash
```

Пример использования (три канала):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Краткий формат вывода по умолчанию. Каждая строка начинается только с приоритета и тега. Подходит для беглого просмотра, но без отметки времени трудно упорядочить события.

Пример использования:

```
adb logcat -v brief
```

Пример вывода:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Добавляет дату и время. Гораздо полезнее краткого формата, потому что видно, когда произошло каждое событие. После секунд показываются миллисекунды.

Пример использования:

```
adb logcat -v time
```

Пример вывода:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

Самый рекомендуемый формат вывода. Включает дату, время, приоритет, тег, PID (идентификатор процесса) и TID (идентификатор потока). В многопоточных приложениях он незаменим, чтобы понять, какой поток что сделал. Этот формат по умолчанию использует и Android Studio.

Пример использования:

```
adb logcat -v threadtime
```

Пример вывода:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Даёт подробный вывод со всеми полями метаданных и пустой строкой между записями. В долгих сеансах показывает каждую запись отдельным блоком, но занимает место.

Пример использования:

```
adb logcat -v long
```

Пример вывода:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Раскрашивает журналы по приоритету. Ошибки — красным, предупреждения — оранжевым, информация — зелёным, отладка — синим. В длинном потоке взгляд сразу падает на ошибку. Терминал должен поддерживать цвета.

Пример использования:

```
adb logcat -v color
```

Пример использования (вместе с time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Добавляет год к формату времени. Используется в длинных журналах или архивах, охватывающих больше одного года. В обычной разработке, как правило, не нужен.

Пример использования:

```
adb logcat -v year
```

Пример вывода:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Показывает идентификатор пользователя (UID) приложения, создавшего запись. Полезно, когда несколько приложений используют один и тот же тег. Это модификатор, поэтому он сочетается с другими форматами.

Пример использования:

```
adb logcat -v uid
```

Пример вывода:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Фильтр По Тегу И Приоритету

Показывает журналы определённого тега начиная с определённого приоритета. Приоритеты: V (Verbose) — самый низкий, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) и S (Silent) — самый высокий. Такая фильтрация — самая мощная возможность logcat.

Пример использования (только ошибки):

```
adb logcat *:E
```

Пример использования (предупреждения и выше):

```
adb logcat *:W
```

Пример использования (белый список):

```
adb logcat ActivityManager:I MyApp:D *:S
```

Последний пример показывает ActivityManager начиная с info и MyApp начиная с debug, а все остальные теги заглушает. В некоторых оболочках символ * особый, поэтому его нужно заключать в кавычки:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Тег

Сокращение для «тихого» фильтра. Показывает только указанный тег и заглушает всё остальное. Самый удобный способ сосредоточиться на собственном теге при разработке.

Пример использования:

```
adb logcat -s MyApp
```

Пример использования (несколько тегов):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

В примере выше проходят только журналы трёх тегов; всё остальное молчит.

---

24- Adb Logcat -S С Приоритетом

Позволяет указать тег и приоритет вместе, например чтобы видеть только предупреждения и выше для тега MyApp.

Пример использования:

```
adb logcat MyApp:W *:S
```

Эта команда показывает только записи warning, error и fatal с тегом MyApp.

---

25- Adb Logcat Фильтр --Pid

Показывает только журналы определённого идентификатора процесса (PID). Чтобы узнать PID приложения, сначала используют adb shell pidof, а затем фильтруют по этому PID. Идеально, чтобы выделить все журналы одного сеанса.

Пример использования (получить PID):

```
adb shell pidof com.example.app
```

Пример вывода:

```
5678
```

Пример использования (фильтр):

```
adb logcat --pid=5678
```

Пример использования (одной строкой):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Время

Показывает журналы начиная с указанной отметки времени, то есть «принеси то, что было после этого момента». Пишется с заглавной T; строчная t означает другое.

Пример использования:

```
adb logcat -T "09-21 14:00:00.000"
```

Пример использования (с сегодняшнего дня):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Число Строк

Со строчной t показывает указанное число последних строк, и команда завершается. Ведёт себя как -d, но выдаёт только последние N строк. Идеально для беглого просмотра.

Пример использования (последние 100 строк):

```
adb logcat -t 100
```

Пример использования (последние 500 строк и фильтр):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Ищет по содержимому с помощью регулярного выражения (regex) и показывает совпадающие строки. Очень мощно для быстрого поиска текста; особенно при поиске нескольких слов сразу это быстрее фильтрации.

Пример использования (одно слово):

```
adb logcat -e "Login"
```

Пример использования (несколько слов):

```
adb logcat -e "Login|Error|Crash"
```

Пример использования (с именем пакета):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Двоичный Вывод

Записывает журналы в двоичном формате. Человек не может их прочитать, но другой logcat может прочитать их снова, а другие инструменты — обработать. Используется в системах автоматического анализа.

Пример использования:

```
adb logcat -B > Ham_Log.Bin
```

Позже этот файл читают так:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Статистика

Показывает статистику журналов: сколько записей создал каждый тег и какие теги вызвали предупреждение «chatty». Если вам кажется, что приложение засыпает журнал записями, эта команда это подтвердит.

Пример использования:

```
adb logcat -S
```

Пример вывода:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R И -N Вместе

Для ротации журналов. -r задаёт размер каждого файла, а -n — сколько файлов хранить. Полезно, когда приложение будет работать долго и нужно вести журнал непрерывно. Старые файлы удаляются, открываются новые.

Пример использования:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

В примере выше, когда файл достигает 4 MiB, открывается новый, и всего хранится 10 файлов. Так диск никогда не заполняется бесконечно.

---

32- Adb Logcat -P Скрипт Фильтра

Задаёт для logcat собственный скрипт фильтра. Это формат фильтра, присущий Android, и им обычно пользуются системные разработчики. Разработчики приложений применяют его редко.

Пример использования:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Отметки Времени

Параметр, который в некоторых форматах добавляет отметку времени в начало каждой строки. Поведение зависит от версии Android. Обычно делает то же, что и -v time, но по-другому.

Пример использования:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Показывает каждую строку в формате printf, то есть приоритет (PID) тег: сообщение. Нужен для совместимости со старыми инструментами, но в современном Android требуется редко.

Пример использования:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Показывает только «сырое» сообщение без метаданных. Пишется лишь сама запись, без тега, приоритета и отметки времени. Полезно, чтобы передать вывод другой программе.

Пример использования:

```
adb logcat -v raw
```

Пример вывода:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Пишет в начале каждой строки только тег, а затем остальное. Приоритет и PID скрыты. Удобно, когда вы сосредоточены на одном теге.

Пример использования:

```
adb logcat -v tag
```

Пример вывода:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Пишет в начале каждой строки сначала PID, потом тег. Удобно для анализа по процессам; сразу видно, какой процесс что записал.

Пример использования:

```
adb logcat -v process
```

Пример вывода:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Пишет в начале каждой строки только идентификатор потока (TID), а затем остальное. При многопоточном анализе показывает, какой поток активен. Краткий формат.

Пример использования:

```
adb logcat -v thread
```

Пример вывода:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D И -T Вместе

Получает журналы за промежуток времени, например за последний час, чтобы сохранить их в файл.

Пример использования:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Пример использования (в файл):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C И -D По Очереди

Поток, который сначала очищает, а потом собирает. Например, очистить перед началом теста и собрать журналы в конце теста.

Пример использования:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat С Сохранением В Файл И Переносом На Компьютер

Удобно для долгой записи в мобильном терминале. Сначала журнал пишут в файл на устройстве, а затем переносят на компьютер с помощью adb pull.

Пример использования (сохранить на устройстве):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Пример использования (перенести на компьютер):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat С Grep

Используется в мобильном терминале (Termux) для быстрого поиска текста через grep. Может быть гибче фильтра regex, потому что доступны все параметры grep.

Пример использования:

```
adb logcat | grep "Hata"
```

Пример использования (без учёта регистра):

```
adb logcat | grep -I "error"
```

Пример использования (с номерами строк):

```
adb logcat | grep -N "Exception"
```

Пример использования (несколько слов):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat С Tail

Сокращает поток до последних строк, когда нужно увидеть только конец, а не весь журнал.

Пример использования:

```
adb logcat | tail -100
```

Пример использования (непрерывно следить):

```
adb logcat | tail -F
```

---

44- Adb Logcat С Head

Показывает первые строки. Подходит, чтобы бегло взглянуть на длинный журнал.

Пример использования:

```
adb logcat -d | head -50
```

---

45- Adb Logcat С Wc

Показывает общее число строк, например чтобы измерить, сколько записей создал тест.

Пример использования:

```
adb logcat -d | wc -l
```

Пример использования (посчитать только ошибки):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat С Sort

Сортирует журналы по алфавиту или по числам. Используется в основном при анализе.

Пример использования:

```
adb logcat -d | sort
```

Пример использования (в обратном порядке):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat С Uniq

Убирает повторяющиеся строки. Если приложение постоянно пишет одну и ту же ошибку, эта команда упрощает вывод.

Пример использования:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

Пример выше подсчитывает и показывает 20 самых частых строк. Очень полезно при отладке.

---

48- Adb Logcat С Awk

Извлекает нужные столбцы или обрабатывает текст, например столбец PID в формате threadtime.

Пример использования (только столбец PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Пример использования (только тег и сообщение):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat С Xargs

Выполняет команду для каждой строки, например отправляет уведомление на каждую строку с ошибкой. Используется в основном в автоматизации.

Пример использования:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat С Sed

Заменяет текст. Идеально, чтобы изменить определённые шаблоны в выводе.

Пример использования:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

Пример выше пишет Uygulama вместо каждого MyApp.

---

51- Adb Logcat С Tee

Одновременно показывает на экране и записывает в файл. Наблюдение в реальном времени и запись идут вместе.

Пример использования:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat С Watch

Получает журналы через равные промежутки времени, например сводку каждые 5 секунд.

Пример использования:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat С Timeout

Останавливает logcat через заданное время. Очень полезно в автоматических тестах.

Пример использования:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat С Alias

Сокращает часто используемые команды. В Termux добавляется в файл .bashrc или .zshrc.

Пример использования (задать alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Затем достаточно вызвать его так:

```
lce
```

---

55- Adb Logcat С Несколькими Устройствами

Выбирает конкретное устройство, когда подключено несколько. Сначала список устройств выводят командой adb devices, затем одно выбирают с помощью -s.

Пример использования:

```
adb devices
```

Пример вывода:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Пример использования (конкретное устройство):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat С Беспроводной Отладкой

Для беспроводной отладки. Когда устройство в той же сети Wi-Fi, подключение идёт без кабеля. Поддерживается начиная с Android 11.

Пример использования (включите на устройстве беспроводную отладку, затем):

```
adb pair 192.168.1.42:37123
```

Пример использования (подключение):

```
adb connect 192.168.1.42:5555
```

Пример использования (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat С Bugreport

Создаёт подробный отчёт обо всей системе. Помимо logcat запускается множество инструментов вроде dumpsys, dmesg и top, и всё собирается в один zip. Самый полный способ отладки.

Пример использования:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Пример использования (перенести на компьютер):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat С Dumpsys

Показывает рядом с logcat текущее состояние системных служб, например что именно сейчас делает ActivityManager.

Пример использования:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat С Трассировками ANR

Получает трассировки стека, когда приложение не отвечает (ANR). Записи ANR хранятся в файле /Data/Anr/Traces.Txt.

Пример использования:

```
adb shell cat /Data/Anr/Traces.Txt
```

Пример использования (нужен root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat С Tombstone-Файлами

Для нативных сбоев. В папке /Data/Tombstones/ на каждый сбой создаётся файл. Сюда смотрят, когда ошибка в коде на C++ или Rust.

Пример использования:

```
adb shell ls /Data/Tombstones/
```

Пример использования (последний tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat С Systrace

Для трассировки производительности всей системы. Собирает данные трассировки на уровне ядра, а не только журналы. Используется вместе с Perfetto.

Пример использования:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat С Layout Inspector

Анализирует разметку приложения на экране. Используется с Layout Inspector, но связанные журналы можно отслеживать и через logcat.

Пример использования:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat С Memory Profiler

Следит за использованием памяти. Memory Profiler обычно находится в Android Studio, но журналы, связанные с памятью, видны и в logcat.

Пример использования:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat С CPU Profiler

Следит за загрузкой процессора. Для простого замера подойдёт и команда top.

Пример использования:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat С Network Profiler

Следит за сетевым трафиком. Библиотеки вроде OkHttp и Retrofit пишут в logcat, но для полного анализа нужен Network Profiler.

Пример использования:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat С GPU Profiler

Следит за загрузкой GPU. Для простого замера подойдёт и команда gfxinfo.

Пример использования:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat С Battery Historian

Следит за расходом батареи. Его можно посмотреть и через dumpsys battery.

Пример использования:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Для Диагностики Wi-Fi

Нужен для отладки проблем с Wi-Fi, например обрывов связи или слабого сигнала.

Пример использования:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Для Диагностики Bluetooth

Нужен для отладки проблем с Bluetooth, например ошибок сопряжения или разрывов связи.

Пример использования:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Для Диагностики Датчиков

Нужен для отладки проблем с датчиками — акселерометром, гироскопом и магнитометром.

Пример использования:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Для Диагностики Камеры

Нужен для отладки проблем с камерой, например при съёмке фото или записи видео.

Пример использования:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Для Диагностики Звука

Нужен для отладки проблем со звуком, например когда звука нет или не работает микрофон.

Пример использования:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Для Диагностики Геолокации

Нужен для отладки проблем с местоположением, например когда нет сигнала GPS или местоположение не обновляется.

Пример использования:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Для Диагностики Уведомлений

Нужен для отладки проблем с уведомлениями, например когда они не появляются или приходят не вовремя.

Пример использования:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Для Диагностики Разрешений

Нужен для отладки проблем с разрешениями, например когда разрешение не выдано или отклонено.

Пример использования:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Для Диагностики WebView

Нужен для отладки проблем с WebView, например когда страница не загружается или возникает ошибка JavaScript.

Пример использования:

```
adb logcat -s Chromium WebView
```

Пример использования (подробнее):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Для Диагностики OkHttp

Нужен для отладки проблем с OkHttp: сетевых запросов, ответов и ошибок.

Пример использования:

```
adb logcat -s Okhttp
```

Пример использования (все уровни):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Для Диагностики Retrofit

Нужен для отладки проблем с Retrofit. Поскольку он обычно работает поверх OkHttp, смотрят журналы OkHttp.

Пример использования:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Для Диагностики Glide

Нужен для отладки проблем с Glide (библиотека загрузки изображений), например когда изображения не загружаются или возникают проблемы с памятью.

Пример использования:

```
adb logcat -s Glide
```

---

80- Adb Logcat Для Диагностики Picasso

Нужен для отладки проблем с Picasso (библиотека загрузки изображений).

Пример использования:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Для Диагностики Room

Нужен для отладки проблем с Room (библиотека баз данных): ошибок запросов и проблем со схемой.

Пример использования:

```
adb logcat -s Room
```

---

82- Adb Logcat Для Диагностики Firebase

Нужен для отладки проблем с Firebase в таких модулях, как Crashlytics, Analytics и Messaging.

Пример использования:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Для Диагностики Crashlytics

Нужен для отладки проблем с Firebase Crashlytics.

Пример использования:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Для Диагностики Analytics

Нужен для отладки проблем аналитики: неотправленных событий и проблем с пакетной отправкой.

Пример использования:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat С Беспроводной Отладкой ADB

Беспроводная отладка иногда требует перезапустить adb. Сначала подключитесь по USB, затем переключитесь на беспроводное подключение.

Пример использования:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat При Перезагрузке Устройства

Перезагружает устройство и следит за журналами, особенно за ошибками, которые возникают при загрузке.

Пример использования (два терминала):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat При Перезапуске Приложения

Перезапускает одно приложение и следит за журналами.

Пример использования:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat При Установке Приложения

Устанавливает APK и следит за журналами.

Пример использования:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat При Удалении Приложения

Удаляет приложение и следит за журналами.

Пример использования:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Со Снимком Экрана

Делает снимок экрана и сохраняет журнал в момент ошибки.

Пример использования:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat С Записью Экрана

Записывает экран и сохраняет журнал в момент ошибки. Поддерживается начиная с Android 4.4.

Пример использования:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat С Тестом Monkey

Проверяет приложение, имитируя случайные действия пользователя. Очень эффективно вместе с журналами.

Пример использования:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat С Тестом UI Automator

Для тестов автоматизации интерфейса с наблюдением за журналами во время теста.

Пример использования:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat С Тестом Espresso

Для тестов Espresso — части автоматизации интерфейса.

Пример использования:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat С Тестом JUnit

Для модульных тестов. Обычно они запускаются на компьютере, но могут работать и на устройстве.

Пример использования:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Со StrictMode

Находит проблемы производительности приложения. Предупреждает, когда к диску или сети обращаются в главном потоке.

Пример использования:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat С Choreographer

Находит пропущенные кадры. Сюда смотрят, когда анимации дёргаются.

Пример использования:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat С Перерисовкой GPU

Находит проблемы с перерисовкой (overdraw). Используется при анализе графической производительности.

Пример использования:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat С Потоком Отрисовки

Находит проблемы потока отрисовки. В современном Android отрисовка идёт в этом потоке.

Пример использования:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat С Отрисовкой GPU

Находит проблемы отрисовки на GPU. Сюда смотрят после включения профилирования отрисовки GPU.

Пример использования:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat С TextView

Находит проблемы текстовых представлений, например проблемы со шрифтами и ошибки измерения.

Пример использования:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat С RecyclerView

Находит проблемы списков: производительность прокрутки и повторное использование представлений.

Пример использования:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat С ViewPager

Находит проблемы при переходах между страницами.

Пример использования:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat С Фрагментами

Находит проблемы жизненного цикла фрагментов.

Пример использования:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat С Activity

Находит проблемы жизненного цикла activity.

Пример использования:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Со Службами

Находит проблемы фоновых служб.

Пример использования:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat С WorkManager

Находит проблемы WorkManager для фоновых задач.

Пример использования:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat С AlarmManager

Находит проблемы AlarmManager для запланированных задач.

Пример использования:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat С SharedPreferences

Находит проблемы SharedPreferences (простое хранение данных).

Пример использования:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat С SQLite

Находит проблемы SQLite (локальная база данных).

Пример использования:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat С Операциями Над Файлами

Находит проблемы чтения и записи файлов.

Пример использования:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat С Сетевыми Операциями

Находит проблемы сетевых подключений, таких как HTTP, TCP и UDP.

Пример использования:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat С SSL

Находит проблемы с SSL-сертификатами. Сюда смотрят, когда не удаётся установить защищённое соединение.

Пример использования:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat С JSON

Находит проблемы разбора JSON.

Пример использования:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat С XML

Находит проблемы разбора XML.

Пример использования:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat С Изображениями

Находит проблемы загрузки и обработки изображений.

Пример использования:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat С Мультимедиа

Находит проблемы воспроизведения и записи мультимедиа.

Пример использования:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat С ExoPlayer

Находит проблемы ExoPlayer (библиотека медиаплеера).

Пример использования:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat С Chromecast

Находит проблемы Chromecast.

Пример использования:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat С Рекламой

Находит проблемы рекламных библиотек.

Пример использования:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat С Платежами

Находит проблемы платежей Google Play.

Пример использования:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat С Картами

Находит проблемы картографических библиотек.

Пример использования:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat С Аутентификацией

Находит проблемы входа, например с Google Sign-In и Firebase Auth.

Пример использования:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat С Уведомлениями

Находит проблемы отправки и получения уведомлений.

Пример использования:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat С Глубокими Ссылками

Находит проблемы глубоких ссылок (deep link), например при открытии приложения по URL.

Пример использования:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat С Интентами

Находит проблемы интентов при переходе между приложениями.

Пример использования:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat С Широковещательными Сообщениями

Находит проблемы широковещательных сообщений (broadcast) между системой и приложениями.

Пример использования:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat С Контент-Провайдерами

Находит проблемы контент-провайдеров, через которые приложения обмениваются данными.

Пример использования:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat С JobScheduler

Находит проблемы JobScheduler для запланированных фоновых задач.

Пример использования:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat С Режимом Doze

Находит проблемы режима Doze (сна) для фоновой работы, пока устройство спит.

Пример использования:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat С Оптимизацией Батареи

Находит проблемы оптимизации батареи, связанные с фоновыми ограничениями.

Пример использования:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat С Утечками Памяти

Находит утечки памяти. Используется вместе с инструментами вроде LeakCanary.

Пример использования:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat С ANR

Находит проблемы ANR (приложение не отвечает).

Пример использования:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Со Сбоями

Находит сбои.

Пример использования:

```
adb logcat -b crash
```

---

135- Adb Logcat С Нативными Сбоями

Находит нативные сбои (C/C++/Rust).

Пример использования:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat С Анализом Tombstone

Анализирует записи нативных сбоев.

Пример использования:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat С Анализом Systrace

Анализирует записи системной трассировки.

Пример использования:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat С Анализом Perfetto

Современный инструмент системной трассировки, более полный, чем logcat.

Пример использования:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat С Анализом Atrace

Для трассировки на уровне системы; более старая форма Perfetto.

Пример использования:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat С Анализом Traceview

Для профилирования на уровне методов.

Пример использования:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat С Трассировкой Методов

Отслеживает конкретный метод.

Пример использования:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat С Дампом Кучи

Создаёт дамп памяти. Очень полезно при анализе памяти.

Пример использования:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat С Журналом GC

Следит за событиями сборки мусора.

Пример использования:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat С Отладкой JNI

Находит проблемы JNI (Java Native Interface).

Пример использования:

```
adb logcat -s JNI
```

---

145- Adb Logcat С Отладкой ART

Находит проблемы ART (Android Runtime).

Пример использования:

```
adb logcat -s Art
```

---

146- Adb Logcat С Отладкой Dalvik

Находит проблемы виртуальной машины Dalvik. Используется на старых версиях Android.

Пример использования:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat И Проблема Chatty

Когда logcat пишет слишком много записей, он выдаёт предупреждение «chatty» и пропускает строки. Решается фильтром или увеличением буфера.

Пример использования (увеличить буфер):

```
adb logcat -G 16M
```

Пример использования (применить фильтр):

```
adb logcat -s MyApp
```

---

148- Adb Logcat С Цветным Выводом

В длинных потоках журналов цветной вывод меньше утомляет глаза и сразу показывает ошибку.

Пример использования:

```
adb logcat -v color
```

Пример использования (с time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Со Скриптом Bash

Позволяет написать скрипт, который собирает журналы автоматически. На его основе можно построить систему отчётов об ошибках.

Пример скрипта:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat: Итог

Как видно из этого списка, logcat — это не одна команда, а целый набор команд. Вместе с фильтрами, форматами, буферами, файлами и другими инструментами он становится самым мощным оружием отладки. Если вы знаете, какое сочетание использовать в каждой ситуации, поиск ошибки займёт считаные минуты. Запомните: первое правило сеанса отладки всегда одно; сначала очистите журнал командой adb logcat -c, затем воспроизведите ошибку, затем наблюдайте через отфильтрованный logcat.

@@ Подключение Android Debug Bridge
@: Команды Беспроводного Подключения ADB
1- Включение Беспроводной Отладки

Сначала на устройстве нужно включить параметры разработчика. Для этого 7 раз нажмите Настройки → О телефоне → Номер сборки. Затем включите Настройки → Система → Для разработчиков → Отладка по Wi-Fi. Без этого шага ни adb pair, ни adb connect работать не будут.

Пример пути:

```
Настройки → О телефоне → Номер сборки (нажать 7 раз)
Настройки → Система → Для разработчиков → Отладка по Wi-Fi (включено)
```

---

2- Как Узнать IP-Адрес И Порт Устройства

При включённой отладке по Wi-Fi откройте пункт «Подключить устройство с помощью кода подключения». Там появятся IP-адрес устройства и случайный номер порта, например 192.168.1.42:37123. Этот порт нужен только для сопряжения и потом меняется. На том же экране есть и 6-значный код подключения.

Пример экрана:

```
IP-адрес и порт
192.168.1.42:37123

Код подключения по Wi-Fi
482915
```

---

3- Команда Adb Pair

Эта команда выполняет сопряжение с устройством. Её запускают на компьютере или в Termux. Введите IP-адрес и порт сопряжения, которые показаны на устройстве. После запуска терминал спросит Enter pairing code: и будет ждать 6-значный код. Если код верный, появится сообщение Successfully paired.

Пример использования:

```
adb pair 192.168.1.42:37123
```

Пример вывода:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

С неверным кодом команда завершится так:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Другие Способы Передать Код В Adb Pair

Вводить код вручную каждый раз не обязательно. Некоторые версии принимают его прямо параметром, хотя из соображений безопасности это обычно не поддерживается. Но попробовать можно.

Пример использования (одной строкой):

```
adb pair 192.168.1.42:37123 482915
```

Пример использования (через конвейер):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Эти способы зависят от версии. Самый надёжный путь — запустить команду и ввести код вручную.

---

5- На Что Обратить Внимание В Adb Pair

Порт сопряжения и порт подключения — разные. Порт сопряжения обычно случайный и большой, порт подключения обычно 5555. Сопряжение делается один раз; чтобы снова подключиться к тому же устройству, достаточно adb connect. Но если сбросить устройство до заводских настроек или выключить отладку по Wi-Fi, сопряжение сбросится, и снова понадобится adb pair.

Пример сравнения:

```
Сопряжение: adb pair 192.168.1.42:37123   → запрашивает код, один раз
Подключение: adb connect 192.168.1.42:5555 → без кода, каждый раз
```

---

6- Команда Adb Connect

Эта команда подключается к сопряжённому устройству. Введите IP-адрес и порт, показанные на главном экране отладки по Wi-Fi. Порт обычно 5555, но иногда другой. При успешном подключении появится сообщение Connected to.

Пример использования:

```
adb connect 192.168.1.42:5555
```

Пример вывода:

```
Connected to 192.168.1.42:5555
```

Если устройство уже подключено, будет такое сообщение:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Без Порта

В некоторых версиях, если порт не указан, по умолчанию пробуется 5555. Но так бывает не на всех устройствах, поэтому надёжнее указывать порт явно.

Пример использования (порт по умолчанию):

```
adb connect 192.168.1.42
```

Пример использования (явный порт):

```
adb connect 192.168.1.42:5555
```

---

8- Проверка Подключения

После подключения команда adb devices показывает, действительно ли устройство подключено. IP-адрес устройства должен быть в списке с пометкой device. Если написано offline, соединение установлено, но ещё нестабильно; попробуйте чуть позже.

Пример использования:

```
adb devices
```

Пример вывода:

```
List of devices attached
192.168.1.42:5555   device
```

Пример вывода (не подключено):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Команда Adb Disconnect

Разрывает подключение. Соединение adb завершается, даже если устройство остаётся в той же сети. Чтобы подключиться снова, ещё раз выполните adb connect; сопряжение не нужно.

Пример использования (одно устройство):

```
adb disconnect 192.168.1.42:5555
```

Пример вывода:

```
Disconnected 192.168.1.42:5555
```

Пример использования (все устройства):

```
adb disconnect
```

Пример вывода:

```
Disconnected everything
```

---

10- Команда Adb Tcpip

Переводит подключённое по USB устройство в беспроводной режим. Эта команда заставляет устройство слушать указанный порт, обычно 5555. Этот способ чаще используется в версиях до Android 11, где нет adb pair.

Пример использования:

```
adb tcpip 5555
```

Пример вывода:

```
Restarting in TCP mode port: 5555
```

Затем отключите USB-кабель и выполните эту команду:

```
adb connect 192.168.1.42:5555
```

---

11- Команда Adb Usb

Возвращает из беспроводного режима в режим USB, когда нужно снова подключить устройство кабелем.

Пример использования:

```
adb usb
```

Пример вывода:

```
Restarting in USB mode
```

---

12- Adb Kill-Server И Start-Server

Иногда сервер adb зависает или отклоняет подключения. Тогда его нужно остановить и запустить снова. Это сбрасывает все подключения и даёт чистый старт.

Пример использования:

```
adb kill-server
adb start-server
```

Пример вывода:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Затем подключитесь снова:

```
adb connect 192.168.1.42:5555
```

---

13- Одна И Та Же Сеть

Для беспроводного adb устройство и компьютер или телефон должны быть в одной сети Wi-Fi. В разных сетях подключения не будет. Гостевая сеть, разделение 5 GHz и 2,4 GHz, VPN или брандмауэр иногда создают проблемы. Простое решение: подключите оба устройства к одному роутеру и выключите VPN.

Пример проверки:

```
Устройство: Настройки → Wi-Fi → Подключённая сеть (напр. Home_Wifi)
Termux: Настройки → Wi-Fi → Подключённая сеть (напр. Home_Wifi)
```

---

14- Как Найти IP-Адрес Вручную

Чтобы найти IP-адрес устройства, откройте Настройки → Wi-Fi → Подключённая сеть → Сведения о сети. Или посмотрите его в Termux командой ниже. Но IP-адрес со временем может меняться, поэтому надёжнее каждый раз проверять экран отладки по Wi-Fi.

Пример использования (Termux):

```
ifconfig wlan0
```

Пример использования (на устройстве):

```
Настройки → Wi-Fi → Подключённая сеть → IP-адрес
```

---

15- Подключение К Самому Себе Из Termux На Том Же Устройстве

Если Termux работает на том же устройстве, adb уже локальный, но всё равно можно подключиться через localhost командой adb connect. Обычно используется 127.0.0.1:5555. Некоторые устройства это отклоняют; тогда используют локальный IP самого устройства.

Пример использования (localhost):

```
adb connect 127.0.0.1:5555
```

Пример использования (собственный локальный IP):

```
adb connect 192.168.1.42:5555
```

---

16- Выполнение Команд При Нескольких Подключённых Устройствах

Если одновременно подключено несколько устройств, в каждой команде выбирайте устройство параметром -s. Иначе adb завершится ошибкой error: more than one device.

Пример использования:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Когда Подключение Обрывается

Если сигнал Wi-Fi ослабевает, устройство уходит в сон или меняется сеть, подключение обрывается. Тогда устройство пропадает из списка adb devices или отображается как offline. Решение: снова выполнить adb connect. Сопряжение обычно остаётся действительным, и adb pair не нужен.

Пример восстановления:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- После Перезагрузки Устройства

После перезагрузки устройства отладка по Wi-Fi обычно выключается. На некоторых устройствах она остаётся включённой, но сопряжение сбрасывается. Когда устройство загрузится, откройте экран отладки по Wi-Fi, возьмите новый порт и новый код, снова выполните adb pair, а затем adb connect.

Пример последовательности:

```
Устройство перезагружено → включить отладку по Wi-Fi
Новый порт: 42157, новый код: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Частые Ошибки При Сопряжении

Failed: Wrong password → код введён неверно.
Failed to connect: Connection refused → порт неверный или устройство не слушает.
Failed: Pairing rejected → устройство отклонило сопряжение; обновите код.
Timeout → проблема с сетью; проверьте, что устройства в одной сети.

Пример ошибки и решения:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Решение: код на устройстве изменился; возьмите новый код и попробуйте снова.

---

20- Отключение Беспроводного Adb

Выключать отладку по Wi-Fi после работы — хорошая идея для безопасности. Её можно выключить в настройках или перейти в режим USB через adb.

Пример использования:

```
adb usb
adb disconnect
```

Пример использования (в настройках):

```
Настройки → Система → Для разработчиков → Отладка по Wi-Fi (выключено)
```

---

21- Различия Между Версиями Android

Android 11 и новее: сопряжение через adb pair обязательно, используется случайный порт.
Android 10 и старше: в беспроводной режим переходят командой adb tcpip 5555; сопряжения нет.

Пример сравнения:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Замечания О Безопасности

Пока отладка по Wi-Fi включена, злоумышленник в той же сети может попытаться получить доступ к устройству. Поэтому держите функцию выключенной в сетях Wi-Fi, которым не доверяете. Код подключения существует для безопасности, но осторожность всё равно не помешает. Выключайте её после работы.

Пример предосторожности:

```
Когда не используется: Настройки → Для разработчиков → Отладка по Wi-Fi (выключено)
В безопасной сети: включайте только дома или в офисе
```

---

23- Вся Последовательность В Одном Месте

Скрипт ниже показывает все шаги — от нуля до подключения. Достаточно заменить IP и порт на данные своего устройства.

Пример последовательности:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Ожидаемый результат:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Скрипт-Сокращение Для Постоянного Подключения

Если вы подключаетесь часто, можно написать скрипт и каждый раз подключаться одной командой. Например, в Termux можно добавить функцию в файл ~/.bashrc.

Пример функции:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Введите IP:порт и код, показанные на устройстве:"
    read -p "Адрес: " ADRES
    adb pair "$ADRES"
    read -p "Порт подключения (напр. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Использование:

```
wifi_adb
```

---

25- Заключение

Беспроводной adb позволяет отлаживать без USB-кабеля и особенно удобен при работе в Termux на самом телефоне. Две основные команды — adb pair и adb connect; первая выполняет сопряжение один раз, вторая используется при каждом подключении. Остальные (adb devices, adb disconnect, adb tcpip, adb kill-server) поддерживают эти две команды и помогают решать проблемы. Помните правило одной сети, выключайте функцию, когда не пользуетесь ею, и перезапускайте сервер, если что-то пошло не так. Вот и всё.
"#;

const ARABIC: &str = r#"@@ أوامر Logcat
1- Adb Logcat

يرسل إلى الطرفية مباشرةً كل سجلات الجهاز. منذ لحظة تشغيل الأمر يظهر على الشاشة كل سطر سجل ينتجه النظام والتطبيقات والخدمات والنواة. لإيقافه اضغط Ctrl + C على لوحة المفاتيح. هذا الأمر هو أساس التصحيح؛ لكنه دون مرشّح قد يمرّر مئات الأسطر في الثانية فيصعب العثور على ما تحتاجه.

مثال على الاستخدام:

```
adb logcat
```

مثال على المخرجات:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

يعرض السجلات المتراكمة حتى الآن ثم ينتهي الأمر. بدلًا من تدفق مستمر يلتقط صورة لتلك اللحظة. استخدمه عند إعداد تقرير خطأ، أو لحفظ الحالة بعد انهيار، أو عندما تريد جلب السجلات مرة واحدة من سكربت.

مثال على الاستخدام:

```
adb logcat -d
```

مثال على الاستخدام (الحفظ في ملف):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

يُستخدم هذا الأمر كثيرًا في السكربتات التي ترسل التقارير تلقائيًا إلى خادم، لأنه ينتهي من تلقاء نفسه ولا يعلّق السكربت أبدًا.

---

3- Adb Logcat -C

يمسح كل مخازن السجل ويحذف كل ما تراكم حتى تلك اللحظة. إذا شغّلته قبل جلسة تصحيح جديدة فلن ترى إلا السجلات التي تظهر بعدها، ولن تربكك الضوضاء القديمة. عند محاولة إعادة إنتاج خطأ يكون عادةً الخطوة الأولى.

مثال على الاستخدام:

```
adb logcat -c
adb logcat
```

بتشغيل الأمرين أعلاه بالترتيب تحصل على تدفق سجلات نظيف تمامًا. لا يظهر إلا ما يحدث بعد المسح.

---

4- Adb Logcat -G

يعرض حجم مخازن السجل الحالية ومدى امتلائها. يسرد سطرًا بسطر امتلاء كل مخزن وسعته الكلية والمقدار المستخدم. إذا كانت السجلات تمرّ بسرعة كبيرة وتُهمل الإدخالات القديمة (ويسمى ذلك «chatty»)، فانظر هنا أولًا.

مثال على الاستخدام:

```
adb logcat -g
```

مثال على المخرجات:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G الحجم

يغيّر حجم مخزن السجل. مثلًا يمكنك رفع مخزن حجمه 256 KiB إلى 16 MiB. هكذا تُحفظ سجلات أكثر وتُهمل الإدخالات القديمة لاحقًا. مفيد جدًا في الاختبارات الطويلة أو عند تصحيح تطبيق كثير الضوضاء. يعود الإعداد إلى أصله عند إعادة تشغيل الجهاز.

مثال على الاستخدام:

```
adb logcat -G 16M
```

مثال على الاستخدام (لتصغيره):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F ملف

يكتب السجلات في الملف المحدد بدلًا من عرضها في الطرفية. تبقى الطرفية نظيفة ويُحفظ السجل. مسار الملف على الجهاز لا على حاسوب. في طرفية الهاتف (Termux) يُكتب عادةً في /Sdcard/.

مثال على الاستخدام:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

مثال على الاستخدام (مع الإلحاق بالملف):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

في المثال أعلاه يُفتح ملف جديد عندما يبلغ الملف 4 MiB، ويُحتفظ بعشرة ملفات إجمالًا.

---

7- Adb Logcat -B Main

يراقب المخزن الرئيسي للتطبيقات فقط. هنا توجد السجلات التي تكتبها التطبيقات عبر استدعاءات مثل Log.d وLog.i وLog.e. هو مراقَب افتراضيًا أصلًا، لكنه يُذكر صراحةً عندما تريد هذا المخزن وحده. وهو أكثر ما يُنظر إليه أثناء تطوير تطبيق.

مثال على الاستخدام:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

يراقب سجلات خدمات النظام. هنا توجد سجلات خدمات مثل ActivityManager وWindowManager وPackageManager وPowerManager. يفيد في رؤية أحداث مثل بدء التطبيقات ومنح الأذونات والارتباط بالخدمات. يُراقب عادةً مع main.

مثال على الاستخدام:

```
adb logcat -b system
```

مثال على المخرجات:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

يراقب سجلات الانهيار فقط. هنا توجد كتل FATAL EXCEPTION التي تُكتب عندما يُغلق تطبيق قسرًا. عند تصحيح انهيار، يكون النظر أولًا في هذا المخزن أسرع طريق، لأن تتبع المكدس موجود هنا مباشرة.

مثال على الاستخدام:

```
adb logcat -b crash
```

مثال على المخرجات:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

يراقب سجلات الراديو والاتصالات والإشارة واتصال البيانات. مفيد عند التعامل مع شريحة SIM أو التسجيل في الشبكة أو اتصال 4G/5G أو الرسائل النصية أو المكالمات. لا يُحتاج إليه عادةً في تطوير التطبيقات المعتاد.

مثال على الاستخدام:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

يراقب السجلات الثنائية لأحداث النظام. لا يحتوي هذا المخزن على نص عادي بل على سجلات قصيرة مرمّزة تتركها استدعاءات Log.event. تظهر بوسوم مثل am_proc_start وam_activity_launch. يُستخدم في تحليل الأداء.

مثال على الاستخدام:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

يراقب سجلات النواة. يعرض الرسائل التي تكتبها نواة Linux، وهي تشبه عادةً مخرجات الأمر dmesg. هنا تجد أخطاء التعريفات ومشكلات الذاكرة ومعلومات الحرارة وإدارة الطاقة. قد يتطلب صلاحيات root.

مثال على الاستخدام:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

يراقب كل المخازن دفعة واحدة. تمرّ main وsystem وcrash وradio وevents وkernel مختلطة. مفيد عندما تريد رؤية كل شيء، لكن المخرجات تصبح مزدحمة جدًا، لذا يُجمع عادةً مع مرشّح.

مثال على الاستخدام:

```
adb logcat -b all
```

مثال على الاستخدام (مع مرشّح):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B عدة مخازن

يراقب في الوقت نفسه عدة مخازن مفصولة بفواصل، مثلًا لرؤية سجلات التطبيق والانهيارات معًا. باختيار القنوات اللازمة فقط تقلّل المخرجات.

مثال على الاستخدام:

```
adb logcat -b main,crash
```

مثال على الاستخدام (ثلاث قنوات):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

صيغة المخرجات القصيرة الافتراضية. يبدأ كل سطر بالأولوية والوسم فقط. جيدة لنظرة سريعة، لكن دون طابع زمني يصعب ترتيب الأحداث.

مثال على الاستخدام:

```
adb logcat -v brief
```

مثال على المخرجات:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

يضيف التاريخ والوقت. أنفع بكثير من الصيغة القصيرة لأنك ترى متى وقع كل حدث. تظهر الأجزاء من الألف من الثانية بعد الثواني.

مثال على الاستخدام:

```
adb logcat -v time
```

مثال على المخرجات:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

صيغة المخرجات الأكثر توصية. تتضمن التاريخ والوقت والأولوية والوسم وPID (معرّف العملية) وTID (معرّف الخيط). في التطبيقات متعددة الخيوط لا غنى عنها لمعرفة أي خيط فعل ماذا. وهي أيضًا الصيغة التي يستخدمها Android Studio افتراضيًا.

مثال على الاستخدام:

```
adb logcat -v threadtime
```

مثال على المخرجات:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

يعطي مخرجات مفصلة بكل حقول البيانات الوصفية وسطر فارغ بين الإدخالات. في الجلسات الطويلة يعرض كل سجل ككتلة، لكنه يشغل مساحة.

مثال على الاستخدام:

```
adb logcat -v long
```

مثال على المخرجات:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

يلوّن السجلات حسب الأولوية. الأخطاء بالأحمر والتحذيرات بالبرتقالي والمعلومات بالأخضر والتصحيح بالأزرق. في تدفق طويل تذهب العين مباشرة إلى الخطأ. يجب أن تدعم الطرفية الألوان.

مثال على الاستخدام:

```
adb logcat -v color
```

مثال على الاستخدام (مع time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

يضيف السنة إلى صيغة الوقت. يُستخدم في السجلات الطويلة أو الأرشيفات التي تمتد لأكثر من سنة. لا يُحتاج إليه عادةً في التطوير المعتاد.

مثال على الاستخدام:

```
adb logcat -v year
```

مثال على المخرجات:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

يعرض معرّف المستخدم (UID) للتطبيق الذي أنتج السجل. مفيد عندما تستخدم عدة تطبيقات الوسم نفسه. هو معدِّل، لذا يُجمع مع الصيغ الأخرى.

مثال على الاستخدام:

```
adb logcat -v uid
```

مثال على المخرجات:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat مرشّح الوسم والأولوية

يعرض سجلات وسم معيّن ابتداءً من أولوية معيّنة. الأولويات هي: V (Verbose) الأدنى، ثم D (Debug) وI (Info) وW (Warning) وE (Error) وF (Fatal) وS (Silent) الأعلى. هذه الطريقة في الترشيح هي أقوى ميزة في logcat.

مثال على الاستخدام (الأخطاء فقط):

```
adb logcat *:E
```

مثال على الاستخدام (التحذيرات فما فوق):

```
adb logcat *:W
```

مثال على الاستخدام (قائمة سماح):

```
adb logcat ActivityManager:I MyApp:D *:S
```

يعرض المثال الأخير ActivityManager ابتداءً من info وMyApp ابتداءً من debug، ويُسكت كل الوسوم الأخرى. في بعض الأصداف يكون المحرف * خاصًا، لذا يجب وضعه بين علامتي اقتباس:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S الوسم

اختصار للمرشّح الصامت. يعرض الوسم المحدد فقط ويُسكت كل ما عداه. أسهل طريقة للتركيز على وسمك أثناء التطوير.

مثال على الاستخدام:

```
adb logcat -s MyApp
```

مثال على الاستخدام (عدة وسوم):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

في المثال أعلاه تمرّ سجلات ثلاثة وسوم فقط؛ وكل ما عداها صامت.

---

24- Adb Logcat -S مع الأولوية

يُستخدم لتحديد الوسم والأولوية معًا، مثلًا لرؤية التحذيرات فما فوق من الوسم MyApp فقط.

مثال على الاستخدام:

```
adb logcat MyApp:W *:S
```

يعرض هذا الأمر سجلات warning وerror وfatal ذات الوسم MyApp فقط.

---

25- Adb Logcat مرشّح --Pid

يعرض سجلات معرّف عملية (PID) معيّن فقط. لمعرفة PID تطبيق ما يُستخدم أولًا adb shell pidof ثم يُرشَّح بذلك الـ PID. مثالي لعزل كل سجلات جلسة واحدة.

مثال على الاستخدام (جلب الـ PID):

```
adb shell pidof com.example.app
```

مثال على المخرجات:

```
5678
```

مثال على الاستخدام (الترشيح):

```
adb logcat --pid=5678
```

مثال على الاستخدام (في سطر واحد):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T الوقت

يعرض السجلات ابتداءً من الطابع الزمني المحدد، أي «هات ما جاء بعد هذه اللحظة». يُكتب بحرف T كبير؛ أما t الصغير فله معنى آخر.

مثال على الاستخدام:

```
adb logcat -T "09-21 14:00:00.000"
```

مثال على الاستخدام (ابتداءً من اليوم):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T عدد الأسطر

بحرف t الصغير يعرض العدد المحدد من أحدث الأسطر ثم ينتهي الأمر. يعمل مثل -d لكنه يعطي آخر N أسطر فقط. مثالي لنظرة سريعة.

مثال على الاستخدام (آخر 100 سطر):

```
adb logcat -t 100
```

مثال على الاستخدام (آخر 500 سطر مع مرشّح):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

يبحث في المحتوى بتعبير نمطي (regex) ويعرض الأسطر المطابقة. قوي جدًا للبحث السريع في النص؛ وخاصة عند البحث عن عدة كلمات دفعة واحدة يكون أسرع من الترشيح.

مثال على الاستخدام (كلمة واحدة):

```
adb logcat -e "Login"
```

مثال على الاستخدام (عدة كلمات):

```
adb logcat -e "Login|Error|Crash"
```

مثال على الاستخدام (مع اسم الحزمة):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B مخرجات ثنائية

يكتب السجلات بصيغة ثنائية. لا يستطيع البشر قراءتها، لكن logcat آخر يمكنه قراءتها من جديد أو يمكن لأدوات أخرى معالجتها. يُستخدم في أنظمة التحليل الآلي.

مثال على الاستخدام:

```
adb logcat -B > Ham_Log.Bin
```

لاحقًا يُقرأ ذلك الملف هكذا:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S الإحصاءات

يعرض إحصاءات السجلات: كم سجلًا أنتج كل وسم، وأي الوسوم أطلقت تحذير «chatty». إذا ظننت أن تطبيقًا يُغرق السجل، فهذا الأمر يؤكد ذلك.

مثال على الاستخدام:

```
adb logcat -S
```

مثال على المخرجات:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R و-N معًا

للسجلات الدوّارة. يحدد -r حجم كل ملف ويحدد -n عدد الملفات المحفوظة. مفيد عندما يعمل تطبيق مدة طويلة وتريد التسجيل باستمرار. تُحذف الملفات القديمة وتُفتح ملفات جديدة.

مثال على الاستخدام:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

في المثال أعلاه يُفتح ملف جديد كلما بلغ ملف 4 MiB، ويُحتفظ بعشرة ملفات إجمالًا. هكذا لا يمتلئ القرص بلا حدود.

---

32- Adb Logcat -P سكربت المرشّح

يحدد سكربت مرشّح مخصصًا لـ logcat. هي صيغة المرشّح الخاصة بـ Android ويستخدمها عادةً مطوّرو النظام. نادرًا ما يستخدمها مطوّرو التطبيقات.

مثال على الاستخدام:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L الطوابع الزمنية

خيار يضيف في بعض الصيغ طابعًا زمنيًا إلى بداية كل سطر. يختلف السلوك بين إصدارات Android. يفعل عادةً ما يفعله -v time لكن بطريقة أخرى.

مثال على الاستخدام:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

يعرض كل سطر بصيغة printf، أي الأولوية (PID) الوسم: الرسالة. يُستخدم للتوافق مع الأدوات القديمة، لكنه نادرًا ما يلزم في Android الحديث.

مثال على الاستخدام:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

يعرض الرسالة الخام فقط دون بيانات وصفية. يُكتب السجل وحده دون وسم أو أولوية أو طابع زمني. مفيد لتمرير المخرجات إلى برنامج آخر.

مثال على الاستخدام:

```
adb logcat -v raw
```

مثال على المخرجات:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

يكتب الوسم فقط في بداية كل سطر ثم الباقي. تُخفى الأولوية والـ PID. عملي عندما تركّز على وسم واحد.

مثال على الاستخدام:

```
adb logcat -v tag
```

مثال على المخرجات:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

يكتب في بداية كل سطر الـ PID أولًا ثم الوسم. عملي للتحليل حسب العملية؛ ترى بسرعة أي عملية كتبت ماذا.

مثال على الاستخدام:

```
adb logcat -v process
```

مثال على المخرجات:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

يكتب معرّف الخيط (TID) فقط في بداية كل سطر ثم الباقي. في التحليل متعدد الخيوط يبيّن أي خيط يعمل. صيغة قصيرة.

مثال على الاستخدام:

```
adb logcat -v thread
```

مثال على المخرجات:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D و-T معًا

يجلب سجلات فترة زمنية، مثلًا سجلات الساعة الأخيرة، لحفظها في ملف.

مثال على الاستخدام:

```
adb logcat -d -T "09-21 13:00:00.000"
```

مثال على الاستخدام (إلى ملف):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C و-D بالتتابع

تدفق يمسح أولًا ثم يجمع. مثلًا المسح قبل بدء اختبار وجمع السجلات في نهايته.

مثال على الاستخدام:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat بالحفظ في ملف ونقله إلى الحاسوب

عملي للتسجيل مدة طويلة في طرفية الهاتف. يُكتب أولًا في ملف على الجهاز ثم يُنقل إلى الحاسوب بالأمر adb pull.

مثال على الاستخدام (الحفظ على الجهاز):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

مثال على الاستخدام (النقل إلى الحاسوب):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat مع Grep

يُستخدم في طرفية الهاتف (Termux) للبحث عن النص بسرعة عبر grep. قد يكون أكثر مرونة من مرشّح regex لأن كل خيارات grep متاحة.

مثال على الاستخدام:

```
adb logcat | grep "Hata"
```

مثال على الاستخدام (دون تمييز حالة الأحرف):

```
adb logcat | grep -I "error"
```

مثال على الاستخدام (مع أرقام الأسطر):

```
adb logcat | grep -N "Exception"
```

مثال على الاستخدام (عدة كلمات):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat مع Tail

يقصر التدفق المباشر على أسطره الأخيرة، عندما تريد رؤية النهاية فقط لا السجل كله.

مثال على الاستخدام:

```
adb logcat | tail -100
```

مثال على الاستخدام (المتابعة المستمرة):

```
adb logcat | tail -F
```

---

44- Adb Logcat مع Head

يعرض الأسطر الأولى. جيد لنظرة سريعة على سجل طويل.

مثال على الاستخدام:

```
adb logcat -d | head -50
```

---

45- Adb Logcat مع Wc

يعرض العدد الإجمالي للأسطر، مثلًا لقياس عدد السجلات التي أنتجها اختبار.

مثال على الاستخدام:

```
adb logcat -d | wc -l
```

مثال على الاستخدام (عدّ الأخطاء فقط):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat مع Sort

يرتّب السجلات أبجديًا أو رقميًا. يُستخدم غالبًا أثناء التحليل.

مثال على الاستخدام:

```
adb logcat -d | sort
```

مثال على الاستخدام (بترتيب عكسي):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat مع Uniq

يزيل الأسطر المكررة. إذا كان تطبيق يكتب الخطأ نفسه باستمرار، فهذا الأمر يبسّط المخرجات.

مثال على الاستخدام:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

يعدّ المثال أعلاه أكثر 20 سطرًا تكرارًا ويعرضها. مفيد جدًا في التصحيح.

---

48- Adb Logcat مع Awk

يستخرج أعمدة معيّنة أو يعالج النص، مثلًا عمود PID في صيغة threadtime.

مثال على الاستخدام (عمود PID فقط):

```
adb logcat -v threadtime | awk '{print $3}'
```

مثال على الاستخدام (الوسم والرسالة فقط):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat مع Xargs

ينفّذ أمرًا لكل سطر، مثلًا إرسال إشعار لكل سطر خطأ. يُستخدم غالبًا في الأتمتة.

مثال على الاستخدام:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat مع Sed

يستبدل النص. مثالي لتغيير أنماط معيّنة في المخرجات.

مثال على الاستخدام:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

يكتب المثال أعلاه Uygulama مكان كل MyApp.

---

51- Adb Logcat مع Tee

يعرض على الشاشة ويكتب في ملف في الوقت نفسه. المراقبة المباشرة والتسجيل يحدثان معًا.

مثال على الاستخدام:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat مع Watch

يجلب السجلات على فترات ثابتة، مثلًا ملخصًا كل 5 ثوانٍ.

مثال على الاستخدام:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat مع Timeout

يوقف logcat بعد مدة محددة. مفيد جدًا في الاختبارات الآلية.

مثال على الاستخدام:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat مع Alias

يختصر الأوامر التي تستخدمها كثيرًا. في Termux يُضاف إلى الملف .bashrc أو .zshrc.

مثال على الاستخدام (تعريف alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

ثم يكفي استدعاؤه هكذا:

```
lce
```

---

55- Adb Logcat مع عدة أجهزة

يستهدف جهازًا معيّنًا عندما تكون عدة أجهزة متصلة. تُسرد الأجهزة أولًا بالأمر adb devices ثم يُختار أحدها بـ -s.

مثال على الاستخدام:

```
adb devices
```

مثال على المخرجات:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

مثال على الاستخدام (جهاز معيّن):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat مع التصحيح اللاسلكي

للتصحيح اللاسلكي. عندما يكون الجهاز على شبكة Wi-Fi نفسها يتم الاتصال دون كابل. مدعوم ابتداءً من Android 11.

مثال على الاستخدام (فعّل التصحيح اللاسلكي على الجهاز، ثم):

```
adb pair 192.168.1.42:37123
```

مثال على الاستخدام (الاتصال):

```
adb connect 192.168.1.42:5555
```

مثال على الاستخدام (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat مع Bugreport

ينشئ تقريرًا مفصلًا عن النظام كله. إلى جانب logcat تُشغَّل أدوات كثيرة مثل dumpsys وdmesg وtop، ويُجمع كل شيء في ملف zip واحد. أشمل طريقة للتصحيح.

مثال على الاستخدام:

```
adb bugreport /Sdcard/Bugreport.Zip
```

مثال على الاستخدام (النقل إلى الحاسوب):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat مع Dumpsys

يعرض إلى جانب logcat الحالة الراهنة لخدمات النظام، مثلًا ما يفعله ActivityManager في تلك اللحظة.

مثال على الاستخدام:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat مع تتبعات ANR

يجلب تتبعات المكدس عندما لا يستجيب تطبيق (ANR). توجد سجلات ANR في الملف /Data/Anr/Traces.Txt.

مثال على الاستخدام:

```
adb shell cat /Data/Anr/Traces.Txt
```

مثال على الاستخدام (يتطلب root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat مع ملفات Tombstone

للانهيارات الأصلية. في المجلد /Data/Tombstones/ يُنشأ ملف لكل انهيار. يُنظر هنا عندما يكون في شيفرة C++ أو Rust خطأ.

مثال على الاستخدام:

```
adb shell ls /Data/Tombstones/
```

مثال على الاستخدام (آخر tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat مع Systrace

لتتبع أداء النظام كله. يجمع بيانات تتبع على مستوى النواة لا السجلات فقط. يُستخدم مع Perfetto.

مثال على الاستخدام:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat مع Layout Inspector

يحلل تخطيط تطبيق على الشاشة. يُستخدم مع Layout Inspector، لكن يمكن متابعة السجلات المرتبطة عبر logcat أيضًا.

مثال على الاستخدام:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat مع Memory Profiler

يراقب استخدام الذاكرة. يوجد Memory Profiler عادةً في Android Studio، لكن السجلات المتعلقة بالذاكرة تظهر في logcat أيضًا.

مثال على الاستخدام:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat مع CPU Profiler

يراقب استخدام المعالج. لقياس بسيط يكفي الأمر top أيضًا.

مثال على الاستخدام:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat مع Network Profiler

يراقب حركة الشبكة. مكتبات مثل OkHttp وRetrofit تكتب في logcat، لكن التحليل الكامل يحتاج إلى Network Profiler.

مثال على الاستخدام:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat مع GPU Profiler

يراقب استخدام معالج الرسوميات. لقياس بسيط يكفي الأمر gfxinfo أيضًا.

مثال على الاستخدام:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat مع Battery Historian

يراقب استهلاك البطارية. يمكن رؤيته أيضًا عبر dumpsys battery.

مثال على الاستخدام:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat لتشخيص Wi-Fi

يُستخدم لتصحيح مشكلات Wi-Fi مثل انقطاع الاتصال أو ضعف الإشارة.

مثال على الاستخدام:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat لتشخيص Bluetooth

يُستخدم لتصحيح مشكلات Bluetooth مثل فشل الاقتران أو انقطاع الاتصال.

مثال على الاستخدام:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat لتشخيص المستشعرات

يُستخدم لتصحيح مشكلات المستشعرات مثل مقياس التسارع والجيروسكوب ومقياس المغناطيسية.

مثال على الاستخدام:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat لتشخيص الكاميرا

يُستخدم لتصحيح مشكلات الكاميرا مثل التقاط الصور أو تسجيل الفيديو.

مثال على الاستخدام:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat لتشخيص الصوت

يُستخدم لتصحيح مشكلات الصوت مثل انعدام الصوت أو عدم عمل الميكروفون.

مثال على الاستخدام:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat لتشخيص الموقع

يُستخدم لتصحيح مشكلات الموقع مثل غياب إشارة GPS أو عدم تحديث الموقع.

مثال على الاستخدام:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat لتشخيص الإشعارات

يُستخدم لتصحيح مشكلات الإشعارات مثل عدم ظهورها أو وصولها في وقت خاطئ.

مثال على الاستخدام:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat لتشخيص الأذونات

يُستخدم لتصحيح مشكلات الأذونات مثل إذن لم يُمنح أو رُفض.

مثال على الاستخدام:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat لتشخيص WebView

يُستخدم لتصحيح مشكلات WebView مثل صفحة لا تُحمَّل أو خطأ JavaScript.

مثال على الاستخدام:

```
adb logcat -s Chromium WebView
```

مثال على الاستخدام (تفاصيل أكثر):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat لتشخيص OkHttp

يُستخدم لتصحيح مشكلات OkHttp: طلبات الشبكة والاستجابات والأخطاء.

مثال على الاستخدام:

```
adb logcat -s Okhttp
```

مثال على الاستخدام (كل المستويات):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat لتشخيص Retrofit

يُستخدم لتصحيح مشكلات Retrofit. ولأنه يعمل عادةً فوق OkHttp تُراجع سجلات OkHttp.

مثال على الاستخدام:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat لتشخيص Glide

يُستخدم لتصحيح مشكلات Glide (مكتبة تحميل الصور) مثل صور لا تُحمَّل أو مشكلات الذاكرة.

مثال على الاستخدام:

```
adb logcat -s Glide
```

---

80- Adb Logcat لتشخيص Picasso

يُستخدم لتصحيح مشكلات Picasso (مكتبة تحميل الصور).

مثال على الاستخدام:

```
adb logcat -s Picasso
```

---

81- Adb Logcat لتشخيص Room

يُستخدم لتصحيح مشكلات Room (مكتبة قواعد البيانات): أخطاء الاستعلام ومشكلات المخطط.

مثال على الاستخدام:

```
adb logcat -s Room
```

---

82- Adb Logcat لتشخيص Firebase

يُستخدم لتصحيح مشكلات Firebase في وحدات مثل Crashlytics وAnalytics وMessaging.

مثال على الاستخدام:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat لتشخيص Crashlytics

يُستخدم لتصحيح مشكلات Firebase Crashlytics.

مثال على الاستخدام:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat لتشخيص Analytics

يُستخدم لتصحيح مشكلات التحليلات: أحداث لم تُرسل ومشكلات الإرسال المجمّع.

مثال على الاستخدام:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat مع تصحيح ADB اللاسلكي

يتطلب التصحيح اللاسلكي أحيانًا إعادة تشغيل adb. اتصل أولًا عبر USB ثم انتقل إلى الاتصال اللاسلكي.

مثال على الاستخدام:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat أثناء إعادة تشغيل الجهاز

يعيد تشغيل الجهاز ويراقب السجلات، خاصة الأخطاء التي تحدث عند الإقلاع.

مثال على الاستخدام (طرفيتان):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat أثناء إعادة تشغيل تطبيق

يعيد تشغيل تطبيق واحد ويراقب السجلات.

مثال على الاستخدام:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat أثناء تثبيت تطبيق

يثبّت ملف APK ويراقب السجلات.

مثال على الاستخدام:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat أثناء إزالة تطبيق

يزيل تطبيقًا ويراقب السجلات.

مثال على الاستخدام:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat مع لقطة شاشة

يلتقط صورة للشاشة ويحفظ السجل لحظة وقوع خطأ.

مثال على الاستخدام:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat مع تسجيل الشاشة

يسجّل الشاشة ويحفظ السجل لحظة وقوع خطأ. مدعوم ابتداءً من Android 4.4.

مثال على الاستخدام:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat مع اختبار Monkey

يختبر تطبيقًا بمحاكاة تفاعلات عشوائية من المستخدم. فعّال جدًا مع السجلات.

مثال على الاستخدام:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat مع اختبار UI Automator

لاختبارات أتمتة الواجهة مع مراقبة السجلات أثناء الاختبار.

مثال على الاستخدام:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat مع اختبار Espresso

لاختبارات Espresso، وهي جزء من أتمتة الواجهة.

مثال على الاستخدام:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat مع اختبار JUnit

لاختبارات الوحدات. تعمل عادةً على حاسوب، لكن يمكن تشغيلها على الجهاز أيضًا.

مثال على الاستخدام:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat مع StrictMode

يكشف مشكلات أداء التطبيق. ينبّه عند الوصول إلى القرص أو الشبكة على الخيط الرئيسي.

مثال على الاستخدام:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat مع Choreographer

يكشف الإطارات المتخطاة. يُنظر هنا عندما تتقطع الحركات.

مثال على الاستخدام:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat مع الرسم الزائد لمعالج الرسوميات

يكشف مشكلات الرسم الزائد (overdraw). يُستخدم في تحليل أداء الرسوميات.

مثال على الاستخدام:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat مع خيط العرض

يكشف مشكلات خيط العرض. في Android الحديث يتم الرسم على هذا الخيط.

مثال على الاستخدام:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat مع عرض معالج الرسوميات

يكشف مشكلات العرض على معالج الرسوميات. يُنظر هنا بعد تفعيل تحليل عرض GPU.

مثال على الاستخدام:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat مع TextView

يكشف مشكلات عروض النص مثل مشكلات الخطوط وأخطاء القياس.

مثال على الاستخدام:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat مع RecyclerView

يكشف مشكلات عروض القوائم: أداء التمرير وإعادة تدوير العروض.

مثال على الاستخدام:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat مع ViewPager

يكشف المشكلات في الانتقال بين الصفحات.

مثال على الاستخدام:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat مع Fragment

يكشف مشكلات دورة حياة الـ Fragment.

مثال على الاستخدام:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat مع Activity

يكشف مشكلات دورة حياة الـ Activity.

مثال على الاستخدام:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat مع الخدمات

يكشف مشكلات الخدمات العاملة في الخلفية.

مثال على الاستخدام:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat مع WorkManager

يكشف مشكلات WorkManager الخاصة بالمهام في الخلفية.

مثال على الاستخدام:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat مع AlarmManager

يكشف مشكلات AlarmManager الخاصة بالمهام المجدولة.

مثال على الاستخدام:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat مع SharedPreferences

يكشف مشكلات SharedPreferences (تخزين بيانات بسيط).

مثال على الاستخدام:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat مع SQLite

يكشف مشكلات SQLite (قاعدة البيانات المحلية).

مثال على الاستخدام:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat مع عمليات الملفات

يكشف مشكلات قراءة الملفات وكتابتها.

مثال على الاستخدام:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat مع عمليات الشبكة

يكشف مشكلات اتصالات الشبكة مثل HTTP وTCP وUDP.

مثال على الاستخدام:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat مع SSL

يكشف مشكلات شهادات SSL. يُنظر هنا عندما تفشل الاتصالات الآمنة.

مثال على الاستخدام:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat مع JSON

يكشف مشكلات تحليل JSON.

مثال على الاستخدام:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat مع XML

يكشف مشكلات تحليل XML.

مثال على الاستخدام:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat مع الصور

يكشف مشكلات تحميل الصور ومعالجتها.

مثال على الاستخدام:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat مع الوسائط

يكشف مشكلات تشغيل الوسائط وتسجيلها.

مثال على الاستخدام:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat مع ExoPlayer

يكشف مشكلات ExoPlayer (مكتبة مشغل الوسائط).

مثال على الاستخدام:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat مع Chromecast

يكشف مشكلات Chromecast.

مثال على الاستخدام:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat مع الإعلانات

يكشف مشكلات مكتبات الإعلانات.

مثال على الاستخدام:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat مع الفوترة

يكشف مشكلات الفوترة في Google Play.

مثال على الاستخدام:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat مع الخرائط

يكشف مشكلات مكتبات الخرائط.

مثال على الاستخدام:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat مع المصادقة

يكشف مشكلات تسجيل الدخول مثل Google Sign-In وFirebase Auth.

مثال على الاستخدام:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat مع الإشعارات

يكشف مشكلات إرسال الإشعارات واستقبالها.

مثال على الاستخدام:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat مع الروابط العميقة

يكشف مشكلات الروابط العميقة (deep link) مثل فتح تطبيق من عنوان URL.

مثال على الاستخدام:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat مع Intent

يكشف مشكلات الـ Intent عند الانتقال بين التطبيقات.

مثال على الاستخدام:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat مع رسائل البث

يكشف مشكلات رسائل البث (broadcast) بين النظام والتطبيقات.

مثال على الاستخدام:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat مع موفّري المحتوى

يكشف مشكلات موفّري المحتوى (content provider) لمشاركة البيانات بين التطبيقات.

مثال على الاستخدام:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat مع JobScheduler

يكشف مشكلات JobScheduler الخاصة بالمهام المجدولة في الخلفية.

مثال على الاستخدام:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat مع وضع Doze

يكشف مشكلات وضع Doze (السكون) الخاصة بالعمل في الخلفية أثناء سكون الجهاز.

مثال على الاستخدام:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat مع تحسين البطارية

يكشف مشكلات تحسين البطارية المتعلقة بقيود الخلفية.

مثال على الاستخدام:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat مع تسرب الذاكرة

يكشف تسربات الذاكرة. يُستخدم مع أدوات مثل LeakCanary.

مثال على الاستخدام:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat مع ANR

يكشف مشكلات ANR (التطبيق لا يستجيب).

مثال على الاستخدام:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat مع الانهيارات

يكشف الانهيارات.

مثال على الاستخدام:

```
adb logcat -b crash
```

---

135- Adb Logcat مع الانهيارات الأصلية

يكشف الانهيارات الأصلية (C/C++/Rust).

مثال على الاستخدام:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat مع تحليل Tombstone

يحلل سجلات الانهيارات الأصلية.

مثال على الاستخدام:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat مع تحليل Systrace

يحلل سجلات تتبع النظام.

مثال على الاستخدام:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat مع تحليل Perfetto

أداة تتبع النظام الحديثة، وهي أشمل من logcat.

مثال على الاستخدام:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat مع تحليل Atrace

للتتبع على مستوى النظام؛ وهو الشكل الأقدم من Perfetto.

مثال على الاستخدام:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat مع تحليل Traceview

للتحليل على مستوى الدوال.

مثال على الاستخدام:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat مع تتبع الدوال

يتتبع دالة معيّنة.

مثال على الاستخدام:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat مع تفريغ الكومة

ينشئ تفريغًا للذاكرة. مفيد جدًا في تحليل الذاكرة.

مثال على الاستخدام:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat مع سجل GC

يراقب أحداث جمع المهملات.

مثال على الاستخدام:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat مع تصحيح JNI

يكشف مشكلات JNI (Java Native Interface).

مثال على الاستخدام:

```
adb logcat -s JNI
```

---

145- Adb Logcat مع تصحيح ART

يكشف مشكلات ART (Android Runtime).

مثال على الاستخدام:

```
adb logcat -s Art
```

---

146- Adb Logcat مع تصحيح Dalvik

يكشف مشكلات آلة Dalvik الافتراضية. يُستخدم في إصدارات Android الأقدم.

مثال على الاستخدام:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat ومشكلة Chatty

عندما يكتب logcat سجلات كثيرة جدًا يعطي تحذير «chatty» ويتخطى أسطرًا. يُحل ذلك بمرشّح أو بمخزن أكبر.

مثال على الاستخدام (تكبير المخزن):

```
adb logcat -G 16M
```

مثال على الاستخدام (استخدام مرشّح):

```
adb logcat -s MyApp
```

---

148- Adb Logcat مع مخرجات ملونة

في تدفقات السجلات الطويلة تريح المخرجات الملونة العين وتُظهر الخطأ فورًا.

مثال على الاستخدام:

```
adb logcat -v color
```

مثال على الاستخدام (مع time):

```
adb logcat -v time,color
```

---

149- Adb Logcat مع سكربت Bash

يُستخدم لكتابة سكربت يجمع السجلات تلقائيًا. ويمكن أن يُبنى عليه نظام للإبلاغ عن الأخطاء.

مثال سكربت:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat خلاصة

كما تبيّن هذه القائمة، ليس logcat أمرًا واحدًا بل مجموعة من الأوامر. ومع المرشّحات والصيغ والمخازن والملفات والأدوات الأخرى يصبح أقوى سلاح في التصحيح. إذا عرفت أي تركيبة تستخدم في كل موقف، فالعثور على خطأ يستغرق دقائق قليلة. تذكّر: القاعدة الأولى لجلسة التصحيح واحدة دائمًا؛ امسح أولًا بـ adb logcat -c، ثم أعد إنتاج الخطأ، ثم راقب بـ logcat مرشَّح.

@@ الاتصال عبر Android Debug Bridge
@: أوامر اتصال ADB اللاسلكي
1- تفعيل التصحيح اللاسلكي

أولًا يجب تفعيل خيارات المطوّر على الجهاز. لذلك اضغط 7 مرات على الإعدادات → حول الهاتف → رقم الإصدار. ثم فعّل الإعدادات → النظام → خيارات المطوّر → تصحيح الأخطاء اللاسلكي. دون هذه الخطوة لن يعمل أي من الأمرين adb pair وadb connect.

مثال على المسار:

```
الإعدادات → حول الهاتف → رقم الإصدار (اضغط 7 مرات)
الإعدادات → النظام → خيارات المطوّر → تصحيح الأخطاء اللاسلكي (مفعّل)
```

---

2- معرفة عنوان IP والمنفذ للجهاز

مع تفعيل التصحيح اللاسلكي افتح خيار «إقران الجهاز باستخدام رمز الإقران». يظهر هناك عنوان IP للجهاز ورقم منفذ عشوائي، مثلًا 192.168.1.42:37123. هذا المنفذ للإقران فقط ويتغير بعد ذلك. وفي الشاشة نفسها يوجد رمز إقران من 6 أرقام.

مثال على الشاشة:

```
عنوان IP والمنفذ
192.168.1.42:37123

رمز الإقران عبر Wi-Fi
482915
```

---

3- الأمر Adb Pair

يقرن هذا الأمر الجهاز. يُشغَّل على حاسوب أو في Termux. اكتب عنوان IP ومنفذ الإقران الظاهرين على الجهاز. عند التشغيل تسأل الطرفية Enter pairing code: وتنتظر الرمز المكوّن من 6 أرقام. إذا كان الرمز صحيحًا تظهر رسالة Successfully paired.

مثال على الاستخدام:

```
adb pair 192.168.1.42:37123
```

مثال على المخرجات:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

مع رمز خاطئ يفشل هكذا:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- طرق أخرى لتمرير الرمز إلى Adb Pair

ليس من الضروري كتابة الرمز يدويًا في كل مرة. بعض الإصدارات تقبله مباشرة كمعامل، مع أن ذلك غير مدعوم عادةً لأسباب أمنية. ومع ذلك يمكنك التجربة.

مثال على الاستخدام (في سطر واحد):

```
adb pair 192.168.1.42:37123 482915
```

مثال على الاستخدام (عبر أنبوب):

```
echo "482915" | adb pair 192.168.1.42:37123
```

تختلف هذه الطرق حسب الإصدار. أوثق طريقة هي تشغيل الأمر وكتابة الرمز يدويًا.

---

5- ما يجب الانتباه إليه في Adb Pair

منفذ الإقران ومنفذ الاتصال مختلفان. منفذ الإقران عادةً عشوائي ومرتفع، ومنفذ الاتصال عادةً 5555. يتم الإقران مرة واحدة فقط؛ وللاتصال بالجهاز نفسه مجددًا يكفي adb connect. لكن إذا أعدت ضبط الجهاز على إعدادات المصنع أو عطّلت التصحيح اللاسلكي، يُمسح الإقران ويلزم adb pair من جديد.

مثال على المقارنة:

```
الإقران: adb pair 192.168.1.42:37123   → يطلب رمزًا، مرة واحدة فقط
الاتصال: adb connect 192.168.1.42:5555  → دون رمز، في كل مرة
```

---

6- الأمر Adb Connect

يتصل هذا الأمر بجهاز مقترن. اكتب عنوان IP والمنفذ الظاهرين في الشاشة الرئيسية للتصحيح اللاسلكي. المنفذ عادةً 5555 لكنه يختلف أحيانًا. إذا نجح الاتصال تظهر رسالة Connected to.

مثال على الاستخدام:

```
adb connect 192.168.1.42:5555
```

مثال على المخرجات:

```
Connected to 192.168.1.42:5555
```

إذا كان متصلًا بالفعل ينبّه هكذا:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect دون منفذ

في بعض الإصدارات، إذا لم تحدد المنفذ يُجرَّب 5555 افتراضيًا. لكن هذا لا يحدث بالطريقة نفسها على كل الأجهزة، لذا فكتابة المنفذ صراحةً هي الأضمن.

مثال على الاستخدام (المنفذ الافتراضي):

```
adb connect 192.168.1.42
```

مثال على الاستخدام (منفذ صريح):

```
adb connect 192.168.1.42:5555
```

---

8- التحقق من الاتصال

بعد الاتصال يبيّن الأمر adb devices هل الجهاز متصل فعلًا. يجب أن يظهر عنوان IP للجهاز في القائمة وبجانبه device. إذا كُتب offline فقد أُنشئ الاتصال لكنه غير مستقر بعد؛ حاول مجددًا بعد قليل.

مثال على الاستخدام:

```
adb devices
```

مثال على المخرجات:

```
List of devices attached
192.168.1.42:5555   device
```

مثال على المخرجات (غير متصل):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- الأمر Adb Disconnect

يقطع الاتصال. ينتهي اتصال adb حتى لو بقي الجهاز على الشبكة نفسها. لإعادة الاتصال شغّل adb connect مرة أخرى؛ لا حاجة إلى الإقران.

مثال على الاستخدام (جهاز واحد):

```
adb disconnect 192.168.1.42:5555
```

مثال على المخرجات:

```
Disconnected 192.168.1.42:5555
```

مثال على الاستخدام (كل الأجهزة):

```
adb disconnect
```

مثال على المخرجات:

```
Disconnected everything
```

---

10- الأمر Adb Tcpip

ينقل جهازًا متصلًا عبر USB إلى الوضع اللاسلكي. يجعل هذا الأمر الجهاز يستمع على المنفذ المحدد، وهو عادةً 5555. هذه طريقة أكثر شيوعًا في الإصدارات السابقة لـ Android 11 التي لا تملك adb pair.

مثال على الاستخدام:

```
adb tcpip 5555
```

مثال على المخرجات:

```
Restarting in TCP mode port: 5555
```

ثم افصل كابل USB وشغّل هذا الأمر:

```
adb connect 192.168.1.42:5555
```

---

11- الأمر Adb Usb

يعود من الوضع اللاسلكي إلى وضع USB، عندما تريد توصيل الجهاز بكابل مرة أخرى.

مثال على الاستخدام:

```
adb usb
```

مثال على المخرجات:

```
Restarting in USB mode
```

---

12- Adb Kill-Server وStart-Server

أحيانًا يتعلّق خادم adb أو يرفض الاتصالات. عندها يجب إيقافه وتشغيله من جديد. هذا يعيد ضبط كل الاتصالات ويعطي بداية نظيفة.

مثال على الاستخدام:

```
adb kill-server
adb start-server
```

مثال على المخرجات:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

ثم اتصل مجددًا:

```
adb connect 192.168.1.42:5555
```

---

13- التواجد على الشبكة نفسها

لكي يعمل adb اللاسلكي يجب أن يكون الجهاز والحاسوب أو الهاتف على شبكة Wi-Fi نفسها. على شبكات مختلفة لا يوجد اتصال. شبكة الضيوف أو الفصل بين 5 GHz و2.4 GHz أو VPN أو جدار حماية قد تسبب مشكلات أحيانًا. الحل البسيط: صِل الجهازين بالموجّه نفسه وأوقف شبكات VPN.

مثال على التحقق:

```
الجهاز: الإعدادات → Wi-Fi → الشبكة المتصلة (مثل Home_Wifi)
Termux: الإعدادات → Wi-Fi → الشبكة المتصلة (مثل Home_Wifi)
```

---

14- معرفة عنوان IP يدويًا

لمعرفة عنوان IP للجهاز اتبع الإعدادات → Wi-Fi → الشبكة المتصلة → تفاصيل الشبكة. أو اعرضه في Termux بالأمر أدناه. لكن عنوان IP قد يتغير مع الوقت، لذا الأوثق أن تتحقق من شاشة التصحيح اللاسلكي في كل مرة.

مثال على الاستخدام (Termux):

```
ifconfig wlan0
```

مثال على الاستخدام (على الجهاز):

```
الإعدادات → Wi-Fi → الشبكة المتصلة → عنوان IP
```

---

15- الاتصال بالجهاز نفسه من Termux

إذا كان Termux يعمل على الجهاز نفسه فـ adb محلي أصلًا، لكن يمكنك مع ذلك الاتصال عبر localhost بالأمر adb connect. يُستخدم عادةً 127.0.0.1:5555. بعض الأجهزة ترفض ذلك؛ وعندها يُستخدم عنوان IP المحلي للجهاز نفسه.

مثال على الاستخدام (localhost):

```
adb connect 127.0.0.1:5555
```

مثال على الاستخدام (عنوان IP المحلي):

```
adb connect 192.168.1.42:5555
```

---

16- تشغيل الأوامر مع عدة أجهزة متصلة

إذا كانت عدة أجهزة متصلة معًا، فاختر الجهاز بالمعامل -s في كل أمر. وإلا يفشل adb برسالة error: more than one device.

مثال على الاستخدام:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- عند انقطاع الاتصال

إذا ضعفت إشارة Wi-Fi أو دخل الجهاز في السكون أو تغيّرت الشبكة، ينقطع الاتصال. عندها يختفي الجهاز من قائمة adb devices أو يظهر offline. الحل: تشغيل adb connect مجددًا. يبقى الإقران عادةً صالحًا ولا حاجة إلى adb pair.

مثال على الاستعادة:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- بعد إعادة تشغيل الجهاز

عند إعادة تشغيل الجهاز يُعطَّل التصحيح اللاسلكي عادةً. في بعض الأجهزة يبقى مفعّلًا لكن الإقران يُمسح. عندما يقلع الجهاز اذهب إلى شاشة التصحيح اللاسلكي وخذ المنفذ الجديد والرمز الجديد، وشغّل adb pair مجددًا ثم adb connect.

مثال على التسلسل:

```
أُعيد تشغيل الجهاز → فعّل التصحيح اللاسلكي
منفذ جديد: 42157، رمز جديد: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- أخطاء شائعة أثناء الإقران

Failed: Wrong password → كُتب الرمز خطأً.
Failed to connect: Connection refused → المنفذ خاطئ أو الجهاز لا يستمع.
Failed: Pairing rejected → رفض الجهاز الإقران؛ جدّد الرمز.
Timeout → توجد مشكلة في الشبكة؛ تأكد أنكما على الشبكة نفسها.

مثال على خطأ وحلّه:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

الحل: تغيّر الرمز على الجهاز؛ خذ الرمز الجديد وحاول مجددًا.

---

20- إيقاف Adb اللاسلكي

إيقاف التصحيح اللاسلكي بعد الانتهاء فكرة جيدة للأمان. يمكنك إيقافه من الإعدادات أو الانتقال إلى وضع USB عبر adb.

مثال على الاستخدام:

```
adb usb
adb disconnect
```

مثال على الاستخدام (من الإعدادات):

```
الإعدادات → النظام → خيارات المطوّر → تصحيح الأخطاء اللاسلكي (معطّل)
```

---

21- الاختلافات حسب إصدار Android

Android 11 وما بعده: الإقران بـ adb pair إلزامي ويُستخدم منفذ عشوائي.
Android 10 وما قبله: يُنتقل إلى الوضع اللاسلكي بـ adb tcpip 5555؛ لا يوجد إقران.

مثال على المقارنة:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- ملاحظات أمنية

ما دام التصحيح اللاسلكي مفعّلًا، قد يحاول شخص سيئ النية على الشبكة نفسها الوصول إلى الجهاز. لذلك أبقِ الميزة معطّلة على شبكات Wi-Fi التي لا تثق بها. رمز الإقران موجود للأمان، لكن الحذر يبقى مفيدًا. أوقفه عند الانتهاء.

مثال على الاحتياط:

```
عند عدم الاستخدام: الإعدادات → خيارات المطوّر → تصحيح الأخطاء اللاسلكي (معطّل)
على شبكة آمنة: فعّله في المنزل أو المكتب فقط
```

---

23- التسلسل كله في مكان واحد

يعرض السكربت أدناه كل الخطوات من الصفر حتى الاتصال. يكفي أن تستبدل عنوان IP والمنفذ بقيم جهازك.

مثال على التسلسل:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

النتيجة المتوقعة:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- سكربت مختصر لاتصال دائم

إذا كنت تتصل كثيرًا، يمكنك كتابة سكربت والاتصال في كل مرة بأمر واحد. مثلًا في Termux يمكنك إضافة دالة إلى الملف ~/.bashrc.

مثال على الدالة:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "أدخل IP:المنفذ والرمز الظاهرين على الجهاز:"
    read -p "العنوان: " ADRES
    adb pair "$ADRES"
    read -p "منفذ الاتصال (مثل 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

الاستخدام:

```
wifi_adb
```

---

25- ملاحظة أخيرة

يتيح adb اللاسلكي التصحيح دون كابل USB، وهو عملي خاصة عند العمل في Termux على الهاتف نفسه. الأمران الأساسيان هما adb pair وadb connect؛ الأول يقرن مرة واحدة، والثاني يُستخدم في كل اتصال. أما البقية (adb devices وadb disconnect وadb tcpip وadb kill-server) فتدعم هذين الأمرين وتساعد في حل المشكلات. لا تنسَ قاعدة الشبكة نفسها، وأوقفه حين لا تستخدمه، وأعد تشغيل الخادم إذا ساء شيء. هذا كل شيء.
"#;

const CHINESE: &str = r#"@@ Logcat 命令
1- Adb Logcat

把设备的全部日志实时输出到终端。从命令运行的那一刻起，系统、应用、服务和内核产生的每一行日志都会出现在屏幕上。要停止，请按键盘上的 Ctrl + C。这个命令是调试的基础；但如果不加过滤，它每秒可能刷出数百行，想找到需要的内容就很困难。

使用示例：

```
adb logcat
```

输出示例：

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

显示目前已积累的日志，然后命令结束。它不是持续的流，而是给那一刻拍一张快照。在准备错误报告、在崩溃后保存状态，或想从脚本中一次性获取日志时使用。

使用示例：

```
adb logcat -d
```

使用示例（保存到文件）：

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

这个命令常用于自动向服务器发送报告的脚本，因为它会自行结束，永远不会卡住脚本。

---

3- Adb Logcat -C

清空所有日志缓冲区，删除到那一刻为止积累的全部内容。如果在新的调试会话之前运行它，你只会看到之后产生的日志，旧的噪音不会干扰你。尝试复现错误时，这通常是第一步。

使用示例：

```
adb logcat -c
adb logcat
```

按顺序运行上面两个命令，就能得到完全干净的日志流。只会出现清空之后发生的事情。

---

4- Adb Logcat -G

显示当前日志缓冲区的大小和已用程度。逐行列出每个缓冲区的占用、总容量和已用量。如果日志滚动太快、旧条目被丢弃（这称为“chatty”），请先看这里。

使用示例：

```
adb logcat -g
```

输出示例：

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G 大小

更改日志缓冲区的大小。例如，可以把 256 KiB 的缓冲区增大到 16 MiB。这样能保存更多日志，旧条目也会更晚被丢弃。在长时间测试或调试噪音很大的应用时非常有用。设备重启后该设置会恢复默认。

使用示例：

```
adb logcat -G 16M
```

使用示例（缩小）：

```
adb logcat -G 64K
```

---

6- Adb Logcat -F 文件

把日志写入指定文件，而不是显示在终端中。终端保持干净，日志也被保存下来。文件路径位于设备上，而不是电脑上。在手机终端（Termux）中通常写到 /Sdcard/。

使用示例：

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

使用示例（追加到文件）：

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

在上面的示例中，文件达到 4 MiB 时会新建一个文件，总共保留 10 个文件。

---

7- Adb Logcat -B Main

只查看应用的主缓冲区。这里是应用通过 Log.d、Log.i、Log.e 等调用写入的日志。默认已经会查看它，但只想看这个缓冲区时要明确指定。开发应用时最常看的就是这里。

使用示例：

```
adb logcat -b main
```

---

8- Adb Logcat -B System

查看系统服务的日志。这里是 ActivityManager、WindowManager、PackageManager、PowerManager 等服务的日志。用于查看应用启动、权限授予、服务绑定等事件。通常与 main 一起查看。

使用示例：

```
adb logcat -b system
```

输出示例：

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

只查看崩溃日志。这里是应用被强制关闭时写入的 FATAL EXCEPTION 块。调试崩溃时，先看这个缓冲区是最快的途径，因为堆栈跟踪就在这里。

使用示例：

```
adb logcat -b crash
```

输出示例：

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

查看无线电、电话、信号和数据连接的日志。处理 SIM 卡、网络注册、4G/5G 连接、短信或通话时很有用。普通应用开发中通常用不到。

使用示例：

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

查看系统事件的二进制日志。这个缓冲区里不是普通文本，而是 Log.event 调用留下的简短编码记录。它们以 am_proc_start、am_activity_launch 等标签出现。用于性能分析。

使用示例：

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

查看内核日志。显示 Linux 内核写出的消息，通常类似 dmesg 命令的输出。这里有驱动错误、内存问题以及温度和电源管理信息。可能需要 root。

使用示例：

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

一次查看所有缓冲区。main、system、crash、radio、events 和 kernel 混在一起输出。想看到全部内容时有用，但输出会非常密集，所以通常与过滤器结合使用。

使用示例：

```
adb logcat -b all
```

使用示例（加过滤器）：

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B 多个缓冲区

同时查看用逗号分隔的多个缓冲区，例如同时看应用日志和崩溃日志。只选需要的通道可以减少输出。

使用示例：

```
adb logcat -b main,crash
```

使用示例（三个通道）：

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

默认的简短输出格式。每行只以优先级和标签开头。适合快速浏览，但没有时间戳就很难排列事件顺序。

使用示例：

```
adb logcat -v brief
```

输出示例：

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

加上日期和时间。比简短格式有用得多，因为能看到每个事件发生的时间。秒后面还会显示毫秒。

使用示例：

```
adb logcat -v time
```

输出示例：

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

最推荐的输出格式。包括日期、时间、优先级、标签、PID（进程标识）和 TID（线程标识）。在多线程应用中，要弄清哪个线程做了什么，它不可或缺。这也是 Android Studio 默认使用的格式。

使用示例：

```
adb logcat -v threadtime
```

输出示例：

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

给出包含所有元数据字段的详细输出，条目之间有空行。在长时间会话中把每条日志显示为一个块，但会占用空间。

使用示例：

```
adb logcat -v long
```

输出示例：

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

按优先级给日志着色。错误为红色，警告为橙色，信息为绿色，调试为蓝色。在长长的日志流中，目光会直接落到错误上。终端必须支持颜色。

使用示例：

```
adb logcat -v color
```

使用示例（与 time 一起）：

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

在时间格式中加上年份。用于跨越一年以上的长日志或归档。普通开发中通常用不到。

使用示例：

```
adb logcat -v year
```

输出示例：

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

显示产生该日志的应用的用户标识（UID）。当多个应用使用同一个标签时很有用。它是一个修饰符，所以要与其他格式组合使用。

使用示例：

```
adb logcat -v uid
```

输出示例：

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat 标签与优先级过滤

显示某个标签从某个优先级开始的日志。优先级依次为：V（Verbose）最低，D（Debug）、I（Info）、W（Warning）、E（Error）、F（Fatal），S（Silent）最高。这种过滤方式是 logcat 最强大的功能。

使用示例（仅错误）：

```
adb logcat *:E
```

使用示例（警告及以上）：

```
adb logcat *:W
```

使用示例（允许列表）：

```
adb logcat ActivityManager:I MyApp:D *:S
```

最后一个示例显示 info 及以上的 ActivityManager 和 debug 及以上的 MyApp，并屏蔽所有其他标签。在某些 shell 中 * 字符有特殊含义，所以必须放在引号里：

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S 标签

静默过滤器的简写。只显示指定的标签，屏蔽其他一切。开发时专注于自己标签的最方便方式。

使用示例：

```
adb logcat -s MyApp
```

使用示例（多个标签）：

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

在上面的示例中只有三个标签的日志能通过；其他全部静默。

---

24- Adb Logcat -S 加优先级

用于同时指定标签和优先级，例如只看 MyApp 标签的警告及以上级别。

使用示例：

```
adb logcat MyApp:W *:S
```

这个命令只显示标签为 MyApp 的 warning、error 和 fatal 日志。

---

25- Adb Logcat --Pid 过滤

只显示某个进程标识（PID）的日志。要知道应用的 PID，先用 adb shell pidof，再按这个 PID 过滤。非常适合把一次会话的所有日志单独分离出来。

使用示例（获取 PID）：

```
adb shell pidof com.example.app
```

输出示例：

```
5678
```

使用示例（过滤）：

```
adb logcat --pid=5678
```

使用示例（一行完成）：

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T 时间

显示从指定时间戳开始的日志，也就是“把这一刻之后的内容拿来”。用大写 T 书写；小写 t 表示别的意思。

使用示例：

```
adb logcat -T "09-21 14:00:00.000"
```

使用示例（从今天开始）：

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T 行数

用小写 t 显示指定数量的最新行，然后命令结束。行为类似 -d，但只给出最后 N 行。非常适合快速浏览。

使用示例（最后 100 行）：

```
adb logcat -t 100
```

使用示例（最后 500 行并过滤）：

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

用正则表达式（regex）搜索内容，并显示匹配的行。对快速文本搜索非常强大；尤其是一次搜索多个词时，比过滤更快。

使用示例（一个词）：

```
adb logcat -e "Login"
```

使用示例（多个词）：

```
adb logcat -e "Login|Error|Crash"
```

使用示例（带包名）：

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B 二进制输出

以二进制格式写出日志。人无法阅读，但另一个 logcat 可以重新读取，其他工具也可以处理。用于自动分析系统。

使用示例：

```
adb logcat -B > Ham_Log.Bin
```

之后这样读取该文件：

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S 统计

显示日志统计：每个标签产生了多少日志，哪些标签触发了“chatty”警告。如果你觉得某个应用在刷屏日志，这个命令可以证实。

使用示例：

```
adb logcat -S
```

输出示例：

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R 与 -N 一起用

用于轮转日志。-r 设置每个文件的大小，-n 设置保留的文件数量。当应用要长时间运行、需要持续记录时很有用。旧文件被删除，新文件被创建。

使用示例：

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

在上面的示例中，每当一个文件达到 4 MiB 就新建一个文件，总共保留 10 个文件。这样磁盘永远不会被无限填满。

---

32- Adb Logcat -P 过滤脚本

为 logcat 指定自定义过滤脚本。这是 Android 特有的过滤格式，通常由系统开发者使用。应用开发者很少使用。

使用示例：

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L 时间戳

在某些格式中为每行开头加上时间戳的选项。行为因 Android 版本而异。通常与 -v time 作用相同，只是方式不同。

使用示例：

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

以 printf 格式显示每一行，即 优先级 (PID) 标签: 消息。用于与旧工具兼容，但在现代 Android 中很少需要。

使用示例：

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

只显示不带元数据的原始消息。只写出日志内容本身，没有标签、优先级或时间戳。适合把输出交给另一个程序。

使用示例：

```
adb logcat -v raw
```

输出示例：

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

每行开头只写标签，然后是其余内容。优先级和 PID 被隐藏。专注于某一个标签时很方便。

使用示例：

```
adb logcat -v tag
```

输出示例：

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

每行开头先写 PID，再写标签。便于按进程分析；能很快看出哪个进程写了什么。

使用示例：

```
adb logcat -v process
```

输出示例：

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

每行开头只写线程标识（TID），然后是其余内容。在多线程分析中显示哪个线程在运行。一种简短的格式。

使用示例：

```
adb logcat -v thread
```

输出示例：

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D 与 -T 一起用

获取某个时间段的日志，例如最近一小时的日志，并保存到文件。

使用示例：

```
adb logcat -d -T "09-21 13:00:00.000"
```

使用示例（写入文件）：

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C 与 -D 先后使用

先清空再收集的流程。例如在测试开始前清空，测试结束时收集日志。

使用示例：

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat 保存到文件并传到电脑

适合在手机终端中长时间记录。先写入设备上的文件，再用 adb pull 传到电脑。

使用示例（保存到设备）：

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

使用示例（传到电脑）：

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat 配合 Grep

在手机终端（Termux）中用 grep 快速搜索文本。它可能比 regex 过滤更灵活，因为 grep 的所有选项都可用。

使用示例：

```
adb logcat | grep "Hata"
```

使用示例（不区分大小写）：

```
adb logcat | grep -I "error"
```

使用示例（带行号）：

```
adb logcat | grep -N "Exception"
```

使用示例（多个词）：

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat 配合 Tail

把实时流缩减为最后几行，适合只想看结尾而不是整个日志的时候。

使用示例：

```
adb logcat | tail -100
```

使用示例（持续跟踪）：

```
adb logcat | tail -F
```

---

44- Adb Logcat 配合 Head

显示最前面的几行。适合快速浏览长日志。

使用示例：

```
adb logcat -d | head -50
```

---

45- Adb Logcat 配合 Wc

显示总行数，例如用来衡量一次测试产生了多少日志。

使用示例：

```
adb logcat -d | wc -l
```

使用示例（只统计错误）：

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat 配合 Sort

按字母或数字对日志排序。主要在分析时使用。

使用示例：

```
adb logcat -d | sort
```

使用示例（倒序）：

```
adb logcat -d | sort -R
```

---

47- Adb Logcat 配合 Uniq

去除重复行。如果应用一直写同一个错误，这个命令能简化输出。

使用示例：

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

上面的示例统计并显示重复最多的 20 行。调试时非常有用。

---

48- Adb Logcat 配合 Awk

提取特定列或处理文本，例如 threadtime 格式中的 PID 列。

使用示例（仅 PID 列）：

```
adb logcat -v threadtime | awk '{print $3}'
```

使用示例（仅标签和消息）：

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat 配合 Xargs

对每一行执行一个命令，例如每出现一行错误就发送一条通知。主要用于自动化。

使用示例：

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat 配合 Sed

替换文本。非常适合修改输出中的特定模式。

使用示例：

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

上面的示例把每个 MyApp 替换为 Uygulama。

---

51- Adb Logcat 配合 Tee

同时显示在屏幕上并写入文件。实时观察和记录同时进行。

使用示例：

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat 配合 Watch

按固定间隔获取日志，例如每 5 秒一次摘要。

使用示例：

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat 配合 Timeout

在设定的时间后停止 logcat。在自动化测试中非常有用。

使用示例：

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat 配合 Alias

缩短常用命令。在 Termux 中添加到 .bashrc 或 .zshrc 文件里。

使用示例（定义 alias）：

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

之后这样调用即可：

```
lce
```

---

55- Adb Logcat 与多台设备

连接了多台设备时指定其中一台。先用 adb devices 列出设备，再用 -s 选择一台。

使用示例：

```
adb devices
```

输出示例：

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

使用示例（指定设备）：

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat 与无线调试

用于无线调试。设备在同一 Wi-Fi 网络中时，无需数据线即可连接。从 Android 11 开始支持。

使用示例（在设备上开启无线调试后）：

```
adb pair 192.168.1.42:37123
```

使用示例（连接）：

```
adb connect 192.168.1.42:5555
```

使用示例（logcat）：

```
adb logcat -v threadtime
```

---

57- Adb Logcat 配合 Bugreport

生成整个系统的详细报告。除 logcat 外，还会运行 dumpsys、dmesg、top 等许多工具，全部打包进一个 zip。最全面的调试方式。

使用示例：

```
adb bugreport /Sdcard/Bugreport.Zip
```

使用示例（传到电脑）：

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat 配合 Dumpsys

在 logcat 旁边显示系统服务的当前状态，例如 ActivityManager 此刻正在做什么。

使用示例：

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat 与 ANR 跟踪

应用无响应（ANR）时获取堆栈跟踪。ANR 记录位于 /Data/Anr/Traces.Txt 文件中。

使用示例：

```
adb shell cat /Data/Anr/Traces.Txt
```

使用示例（需要 root）：

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat 与 Tombstone

用于原生崩溃。每次崩溃都会在 /Data/Tombstones/ 文件夹中生成一个文件。C++ 或 Rust 代码出错时就看这里。

使用示例：

```
adb shell ls /Data/Tombstones/
```

使用示例（最新的 tombstone）：

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat 配合 Systrace

用于全系统的性能跟踪。收集内核级的跟踪数据，而不仅仅是日志。与 Perfetto 一起使用。

使用示例：

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat 配合 Layout Inspector

分析应用在屏幕上的布局。与 Layout Inspector 一起使用，但相关日志也可以通过 logcat 跟踪。

使用示例：

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat 配合 Memory Profiler

观察内存使用。Memory Profiler 通常在 Android Studio 中，但与内存相关的日志在 logcat 中也能看到。

使用示例：

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat 配合 CPU Profiler

观察 CPU 使用。简单测量用 top 命令也可以。

使用示例：

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat 配合 Network Profiler

观察网络流量。OkHttp、Retrofit 等库会写入 logcat，但完整分析需要 Network Profiler。

使用示例：

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat 配合 GPU Profiler

观察 GPU 使用。简单测量用 gfxinfo 命令也可以。

使用示例：

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat 配合 Battery Historian

观察电池消耗。也可以用 dumpsys battery 查看。

使用示例：

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat 诊断 Wi-Fi

用于调试 Wi-Fi 问题，例如连接中断或信号弱。

使用示例：

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat 诊断蓝牙

用于调试蓝牙问题，例如配对失败或断开连接。

使用示例：

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat 诊断传感器

用于调试加速度计、陀螺仪、磁力计等传感器的问题。

使用示例：

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat 诊断相机

用于调试相机问题，例如拍照或录像。

使用示例：

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat 诊断音频

用于调试音频问题，例如没有声音或麦克风不工作。

使用示例：

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat 诊断定位

用于调试定位问题，例如没有 GPS 信号或位置不更新。

使用示例：

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat 诊断通知

用于调试通知问题，例如通知不显示或在错误的时间到达。

使用示例：

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat 诊断权限

用于调试权限问题，例如权限未授予或被拒绝。

使用示例：

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat 诊断 WebView

用于调试 WebView 问题，例如页面无法加载或 JavaScript 错误。

使用示例：

```
adb logcat -s Chromium WebView
```

使用示例（更多细节）：

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat 诊断 OkHttp

用于调试 OkHttp 问题：网络请求、响应和错误。

使用示例：

```
adb logcat -s Okhttp
```

使用示例（所有级别）：

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat 诊断 Retrofit

用于调试 Retrofit 问题。因为它通常基于 OkHttp，所以查看 OkHttp 的日志。

使用示例：

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat 诊断 Glide

用于调试 Glide（图片加载库）的问题，例如图片无法加载或内存问题。

使用示例：

```
adb logcat -s Glide
```

---

80- Adb Logcat 诊断 Picasso

用于调试 Picasso（图片加载库）的问题。

使用示例：

```
adb logcat -s Picasso
```

---

81- Adb Logcat 诊断 Room

用于调试 Room（数据库库）的问题：查询错误和架构问题。

使用示例：

```
adb logcat -s Room
```

---

82- Adb Logcat 诊断 Firebase

用于调试 Firebase 问题，涉及 Crashlytics、Analytics、Messaging 等模块。

使用示例：

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat 诊断 Crashlytics

用于调试 Firebase Crashlytics 的问题。

使用示例：

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat 诊断 Analytics

用于调试分析问题：未发送的事件和批量发送问题。

使用示例：

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat 与 ADB 无线调试

无线调试有时需要重启 adb。先通过 USB 连接，再切换到无线。

使用示例：

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat 在设备重启时

重启设备并观察日志，尤其是启动时发生的错误。

使用示例（两个终端）：

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat 在应用重启时

重启单个应用并观察日志。

使用示例：

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat 在安装应用时

安装 APK 并观察日志。

使用示例：

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat 在卸载应用时

卸载应用并观察日志。

使用示例：

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat 与截屏

截取屏幕，并在出错时保存日志。

使用示例：

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat 与录屏

录制屏幕，并在出错时保存日志。从 Android 4.4 开始支持。

使用示例：

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat 与 Monkey 测试

通过模拟随机的用户操作来测试应用。与日志配合非常有效。

使用示例：

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat 与 UI Automator 测试

用于界面自动化测试，在测试期间观察日志。

使用示例：

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat 与 Espresso 测试

用于 Espresso 测试，它是界面自动化的一个子集。

使用示例：

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat 与 JUnit 测试

用于单元测试。通常在电脑上运行，但也可以在设备上运行。

使用示例：

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat 与 StrictMode

检测应用的性能问题。在主线程访问磁盘或网络时发出警告。

使用示例：

```
adb logcat -s StrictMode
```

---

97- Adb Logcat 与 Choreographer

检测跳帧。动画卡顿时就看这里。

使用示例：

```
adb logcat -s Choreographer
```

---

98- Adb Logcat 与 GPU 过度绘制

检测过度绘制问题。用于图形性能分析。

使用示例：

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat 与渲染线程

检测渲染线程的问题。在现代 Android 中，绘制发生在这个线程上。

使用示例：

```
adb logcat -s RenderThread
```

---

100- Adb Logcat 与 GPU 渲染

检测 GPU 渲染问题。开启 GPU 渲染分析后就看这里。

使用示例：

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat 与 TextView

检测文本视图的问题，例如字体问题和测量错误。

使用示例：

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat 与 RecyclerView

检测列表视图的问题：滚动性能和视图回收。

使用示例：

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat 与 ViewPager

检测页面切换中的问题。

使用示例：

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat 与 Fragment

检测 Fragment 生命周期的问题。

使用示例：

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat 与 Activity

检测 Activity 生命周期的问题。

使用示例：

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat 与服务

检测后台服务的问题。

使用示例：

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat 与 WorkManager

检测 WorkManager 的问题，用于后台任务。

使用示例：

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat 与 AlarmManager

检测 AlarmManager 的问题，用于定时任务。

使用示例：

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat 与 SharedPreferences

检测 SharedPreferences（简单数据存储）的问题。

使用示例：

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat 与 SQLite

检测 SQLite（本地数据库）的问题。

使用示例：

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat 与文件操作

检测文件读写的问题。

使用示例：

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat 与网络操作

检测 HTTP、TCP、UDP 等网络连接的问题。

使用示例：

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat 与 SSL

检测 SSL 证书问题。安全连接失败时就看这里。

使用示例：

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat 与 JSON

检测 JSON 解析问题。

使用示例：

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat 与 XML

检测 XML 解析问题。

使用示例：

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat 与图片

检测图片加载和处理的问题。

使用示例：

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat 与媒体

检测媒体播放和录制的问题。

使用示例：

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat 与 ExoPlayer

检测 ExoPlayer（媒体播放器库）的问题。

使用示例：

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat 与 Chromecast

检测 Chromecast 的问题。

使用示例：

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat 与广告

检测广告库的问题。

使用示例：

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat 与结算

检测 Google Play 结算的问题。

使用示例：

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat 与地图

检测地图库的问题。

使用示例：

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat 与身份验证

检测登录问题，例如 Google Sign-In 和 Firebase Auth。

使用示例：

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat 与通知

检测通知发送和接收的问题。

使用示例：

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat 与深层链接

检测深层链接（deep link）的问题，例如从 URL 打开应用。

使用示例：

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat 与 Intent

检测应用之间跳转时 Intent 的问题。

使用示例：

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat 与广播

检测系统与应用之间广播（broadcast）消息的问题。

使用示例：

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat 与内容提供器

检测内容提供器（content provider）的问题，用于在应用之间共享数据。

使用示例：

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat 与 JobScheduler

检测 JobScheduler 的问题，用于后台定时任务。

使用示例：

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat 与 Doze 模式

检测 Doze（休眠）模式的问题，关乎设备休眠时的后台工作。

使用示例：

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat 与电池优化

检测电池优化的问题，关乎后台限制。

使用示例：

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat 与内存泄漏

检测内存泄漏。与 LeakCanary 等工具一起使用。

使用示例：

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat 与 ANR

检测 ANR（应用无响应）问题。

使用示例：

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat 与崩溃

检测崩溃。

使用示例：

```
adb logcat -b crash
```

---

135- Adb Logcat 与原生崩溃

检测原生崩溃（C/C++/Rust）。

使用示例：

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat 与 Tombstone 分析

分析原生崩溃的记录。

使用示例：

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat 与 Systrace 分析

分析系统跟踪记录。

使用示例：

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat 与 Perfetto 分析

现代的系统跟踪工具，比 logcat 更全面。

使用示例：

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat 与 Atrace 分析

用于系统级跟踪；是 Perfetto 的旧形式。

使用示例：

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat 与 Traceview 分析

用于方法级性能分析。

使用示例：

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat 与方法跟踪

跟踪某个特定方法。

使用示例：

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat 与堆转储

生成内存转储。在内存分析中非常有用。

使用示例：

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat 与 GC 日志

观察垃圾回收事件。

使用示例：

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat 与 JNI 调试

检测 JNI（Java Native Interface）的问题。

使用示例：

```
adb logcat -s JNI
```

---

145- Adb Logcat 与 ART 调试

检测 ART（Android Runtime）的问题。

使用示例：

```
adb logcat -s Art
```

---

146- Adb Logcat 与 Dalvik 调试

检测 Dalvik 虚拟机的问题。用于较旧的 Android 版本。

使用示例：

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat 与 Chatty 问题

logcat 写出的日志过多时，会给出“chatty”警告并跳过一些行。可以用过滤器或更大的缓冲区解决。

使用示例（增大缓冲区）：

```
adb logcat -G 16M
```

使用示例（使用过滤器）：

```
adb logcat -s MyApp
```

---

148- Adb Logcat 彩色输出

在很长的日志流中，彩色输出让眼睛更轻松，也能立刻看到错误。

使用示例：

```
adb logcat -v color
```

使用示例（配合 time）：

```
adb logcat -v time,color
```

---

149- Adb Logcat 与 Bash 脚本

用于编写自动收集日志的脚本。可以在它之上搭建错误报告系统。

示例脚本：

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat 总结

正如这份列表所示，logcat 不是一个命令，而是一组命令。配合过滤器、格式、缓冲区、文件和其他工具，它会成为调试中最强大的武器。如果你知道每种情况下该用哪种组合，找到一个错误只需几分钟。请记住：调试会话的第一条规则永远相同；先用 adb logcat -c 清空，然后复现错误，再用过滤后的 logcat 观察。

@@ Android Debug Bridge 连接
@: ADB 无线连接命令
1- 开启无线调试

首先要在设备上开启开发者选项。为此，请连续点按 设置 → 关于手机 → 版本号 7 次。然后开启 设置 → 系统 → 开发者选项 → 无线调试。不完成这一步，adb pair 和 adb connect 命令都无法使用。

路径示例：

```
设置 → 关于手机 → 版本号（点按 7 次）
设置 → 系统 → 开发者选项 → 无线调试（开启）
```

---

2- 查找设备的 IP 地址和端口

开启无线调试后，打开“使用配对码配对设备”选项。那里会显示设备的 IP 地址和一个随机端口号，例如 192.168.1.42:37123。这个端口只用于配对，之后会改变。同一界面上还有一个 6 位数的配对码。

界面示例：

```
IP 地址和端口
192.168.1.42:37123

WLAN 配对码
482915
```

---

3- Adb Pair 命令

这个命令用于配对设备。在电脑或 Termux 中运行。输入设备上显示的 IP 地址和配对端口。运行后，终端会询问 Enter pairing code: 并等待 6 位数的配对码。如果配对码正确，会出现 Successfully paired 消息。

使用示例：

```
adb pair 192.168.1.42:37123
```

输出示例：

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

配对码错误时会这样失败：

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- 向 Adb Pair 提供配对码的其他方式

不一定每次都要手动输入配对码。有些版本可以直接把它作为参数接收，不过出于安全考虑通常不支持。但可以试一试。

使用示例（一行完成）：

```
adb pair 192.168.1.42:37123 482915
```

使用示例（通过管道）：

```
echo "482915" | adb pair 192.168.1.42:37123
```

这些方式因版本而异。最可靠的方法是运行命令后手动输入配对码。

---

5- 使用 Adb Pair 的注意事项

配对端口和连接端口是不同的。配对端口通常是随机的大数字，连接端口通常是 5555。配对只需一次；再次连接同一台设备时，adb connect 就够了。但如果把设备恢复出厂设置或关闭无线调试，配对会被清除，需要重新执行 adb pair。

对比示例：

```
配对：adb pair 192.168.1.42:37123   → 需要配对码，只需一次
连接：adb connect 192.168.1.42:5555  → 无需配对码，每次都用
```

---

6- Adb Connect 命令

这个命令连接到已配对的设备。输入无线调试主界面上显示的 IP 地址和端口。端口通常是 5555，但有时不同。连接成功时会出现 Connected to 消息。

使用示例：

```
adb connect 192.168.1.42:5555
```

输出示例：

```
Connected to 192.168.1.42:5555
```

如果已经连接，会这样提示：

```
Already connected to 192.168.1.42:5555
```

---

7- 不带端口的 Adb Connect

在某些版本中，如果不指定端口，会默认尝试 5555。但并非所有设备都如此，所以明确写出端口最稳妥。

使用示例（默认端口）：

```
adb connect 192.168.1.42
```

使用示例（明确端口）：

```
adb connect 192.168.1.42:5555
```

---

8- 检查连接

连接后，adb devices 命令会显示设备是否真的已连接。设备的 IP 地址必须出现在列表中，旁边写着 device。如果写着 offline，说明连接已建立但还不稳定；请稍后再试。

使用示例：

```
adb devices
```

输出示例：

```
List of devices attached
192.168.1.42:5555   device
```

输出示例（未连接）：

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Adb Disconnect 命令

断开连接。即使设备仍在同一网络中，adb 连接也会结束。要重新连接，再次运行 adb connect 即可；无需重新配对。

使用示例（一台设备）：

```
adb disconnect 192.168.1.42:5555
```

输出示例：

```
Disconnected 192.168.1.42:5555
```

使用示例（所有设备）：

```
adb disconnect
```

输出示例：

```
Disconnected everything
```

---

10- Adb Tcpip 命令

把通过 USB 连接的设备切换到无线模式。这个命令让设备在指定端口上监听，通常是 5555。这是在没有 adb pair 的 Android 11 之前版本中更常用的方法。

使用示例：

```
adb tcpip 5555
```

输出示例：

```
Restarting in TCP mode port: 5555
```

然后拔掉 USB 数据线并运行这个命令：

```
adb connect 192.168.1.42:5555
```

---

11- Adb Usb 命令

从无线模式切回 USB 模式，适用于想再次用数据线连接设备的时候。

使用示例：

```
adb usb
```

输出示例：

```
Restarting in USB mode
```

---

12- Adb Kill-Server 与 Start-Server

有时 adb 服务器会卡住或拒绝连接。这时需要停止并重新启动它。这样会重置所有连接，重新干净地开始。

使用示例：

```
adb kill-server
adb start-server
```

输出示例：

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

然后重新连接：

```
adb connect 192.168.1.42:5555
```

---

13- 处于同一网络

要使用无线 adb，设备和电脑或手机必须在同一个 Wi-Fi 网络中。不同网络之间无法连接。访客网络、5 GHz 与 2.4 GHz 的划分、VPN 或防火墙有时会造成问题。简单的解决办法：把两台设备连到同一个路由器，并关闭 VPN。

检查示例：

```
设备：设置 → WLAN → 已连接的网络（例如 Home_Wifi）
Termux：设置 → WLAN → 已连接的网络（例如 Home_Wifi）
```

---

14- 手动查找 IP 地址

要查找设备的 IP 地址，请打开 设置 → WLAN → 已连接的网络 → 网络详情。也可以在 Termux 中用下面的命令查看。但 IP 地址可能随时间变化，所以最可靠的做法是每次都查看无线调试界面。

使用示例（Termux）：

```
ifconfig wlan0
```

使用示例（在设备上）：

```
设置 → WLAN → 已连接的网络 → IP 地址
```

---

15- 在同一设备上从 Termux 连接自身

如果 Termux 运行在同一台设备上，adb 本来就是本地的，但仍然可以用 adb connect 通过 localhost 连接。通常使用 127.0.0.1:5555。有些设备会拒绝；这时使用设备自身的本地 IP。

使用示例（localhost）：

```
adb connect 127.0.0.1:5555
```

使用示例（自身的本地 IP）：

```
adb connect 192.168.1.42:5555
```

---

16- 连接多台设备时运行命令

如果同时连接了多台设备，请在每条命令中用 -s 参数选择设备。否则 adb 会以 error: more than one device 失败。

使用示例：

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- 连接断开时

如果 Wi-Fi 信号变弱、设备进入休眠或网络发生变化，连接就会断开。这时设备会从 adb devices 列表中消失，或显示为 offline。解决方法：再次运行 adb connect。配对通常仍然有效，不需要 adb pair。

恢复示例：

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- 设备重启之后

设备重启后，无线调试通常会关闭。有些设备上它仍然开着，但配对会被清除。设备启动完成后，进入无线调试界面，获取新的端口和新的配对码，重新运行 adb pair，然后运行 adb connect。

流程示例：

```
设备已重启 → 开启无线调试
新端口：42157，新配对码：735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- 配对时的常见错误

Failed: Wrong password → 配对码输入错误。
Failed to connect: Connection refused → 端口错误或设备未在监听。
Failed: Pairing rejected → 设备拒绝了配对；请刷新配对码。
Timeout → 网络有问题；请检查是否在同一网络中。

错误示例与解决方法：

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

解决方法：设备上的配对码已变化；获取新的配对码后再试。

---

20- 关闭无线 Adb

用完后关闭无线调试，有利于安全。可以在设置中关闭，也可以用 adb 切换到 USB 模式。

使用示例：

```
adb usb
adb disconnect
```

使用示例（在设置中）：

```
设置 → 系统 → 开发者选项 → 无线调试（关闭）
```

---

21- 不同 Android 版本的差异

Android 11 及以上：必须用 adb pair 配对，并使用随机端口。
Android 10 及以下：用 adb tcpip 5555 切换到无线模式；没有配对。

对比示例：

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- 安全注意事项

无线调试开启期间，同一网络中怀有恶意的人可能会尝试访问设备。所以在不信任的 Wi-Fi 网络中请保持该功能关闭。配对码是为了安全而设的，但依然要小心。用完后请关闭。

防范示例：

```
不使用时：设置 → 开发者选项 → 无线调试（关闭）
在安全网络中：只在家里或办公室开启
```

---

23- 完整流程一览

下面的脚本展示了从零到连接的所有步骤。只需把 IP 和端口换成你自己设备的即可。

流程示例：

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

预期结果：

```
List of devices attached
192.168.1.42:5555   device
```

---

24- 用于持久连接的快捷脚本

如果你经常连接，可以写一个脚本，每次用一条命令完成连接。例如在 Termux 中，可以在 ~/.bashrc 文件里添加一个函数。

函数示例：

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "请输入设备上显示的 IP:端口 和配对码："
    read -p "地址: " ADRES
    adb pair "$ADRES"
    read -p "连接端口（例如 5555）: " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

用法：

```
wifi_adb
```

---

25- 结语

无线 adb 让你无需 USB 数据线即可调试，在手机本身的 Termux 中工作时尤其方便。两个基本命令是 adb pair 和 adb connect；前者只需配对一次，后者每次连接时使用。其余命令（adb devices、adb disconnect、adb tcpip、adb kill-server）用于辅助这两个命令并解决问题。别忘了同一网络这条规则，不用时请关闭，出现问题就重启服务器。就是这样。
"#;

const JAPANESE: &str = r#"@@ Logcat コマンド
1- Adb Logcat

デバイスのすべてのログをリアルタイムでターミナルに出力します。コマンドを実行した瞬間から、システム、アプリ、サービス、カーネルが出力するログの各行が画面に表示されます。止めるにはキーボードで Ctrl + C を押します。このコマンドはデバッグの基本ですが、フィルタなしでは毎秒数百行が流れることもあり、必要な情報を見つけるのが難しくなります。

使用例：

```
adb logcat
```

出力例：

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

これまでに溜まったログを表示して、コマンドが終了します。連続したストリームではなく、その瞬間のスナップショットを撮ります。バグレポートの準備、クラッシュ後の状態保存、スクリプトから一度だけログを取得したいときに使います。

使用例：

```
adb logcat -d
```

使用例（ファイルに保存）：

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

このコマンドは自動でサーバーにレポートを送るスクリプトでよく使われます。自分で終了し、スクリプトを止めてしまうことがないからです。

---

3- Adb Logcat -C

すべてのログバッファを消去し、その時点までに溜まった内容をすべて削除します。新しいデバッグセッションの前に実行すれば、その後に出たログだけが表示され、古いノイズに惑わされません。バグを再現しようとするとき、たいていこれが最初の一歩です。

使用例：

```
adb logcat -c
adb logcat
```

上の 2 つのコマンドを順に実行すると、完全にきれいなログストリームが得られます。消去の後に起きたことだけが表示されます。

---

4- Adb Logcat -G

現在のログバッファのサイズと使用状況を表示します。各バッファの使用量、総容量、使用済みの量を 1 行ずつ一覧にします。ログの流れが速すぎて古いエントリが捨てられる（これを「chatty」と呼びます）ときは、まずここを確認します。

使用例：

```
adb logcat -g
```

出力例：

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G サイズ

ログバッファのサイズを変更します。たとえば 256 KiB のバッファを 16 MiB に増やせます。こうするとより多くのログが保持され、古いエントリが捨てられるのが遅くなります。長時間のテストや、ログの多いアプリのデバッグでとても役立ちます。設定はデバイスを再起動すると元に戻ります。

使用例：

```
adb logcat -G 16M
```

使用例（小さくする）：

```
adb logcat -G 64K
```

---

6- Adb Logcat -F ファイル

ログをターミナルに表示する代わりに、指定したファイルに書き込みます。ターミナルはすっきりしたまま、ログは保存されます。ファイルのパスはパソコンではなくデバイス上にあります。スマホのターミナル（Termux）では通常 /Sdcard/ に書き込みます。

使用例：

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

使用例（ファイルに追記）：

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

上の例では、ファイルが 4 MiB に達すると新しいファイルが開かれ、合計 10 個のファイルが保持されます。

---

7- Adb Logcat -B Main

アプリのメインバッファだけを監視します。ここにはアプリが Log.d、Log.i、Log.e などの呼び出しで書いたログが入ります。デフォルトでも監視されていますが、このバッファだけを見たいときに明示的に指定します。アプリ開発中に最もよく見る場所です。

使用例：

```
adb logcat -b main
```

---

8- Adb Logcat -B System

システムサービスのログを監視します。ここには ActivityManager、WindowManager、PackageManager、PowerManager などのサービスのログが入ります。アプリの起動、権限の付与、サービスへのバインドといったイベントを見るのに使います。通常は main と一緒に監視します。

使用例：

```
adb logcat -b system
```

出力例：

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

クラッシュログだけを監視します。ここにはアプリが強制終了したときに書かれる FATAL EXCEPTION ブロックが入ります。クラッシュをデバッグするとき、まずこのバッファを見るのが一番の近道です。スタックトレースがそのままここにあるからです。

使用例：

```
adb logcat -b crash
```

出力例：

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

無線、電話、電波、データ通信のログを監視します。SIM カード、ネットワーク登録、4G/5G 接続、SMS や通話を扱うときに役立ちます。通常のアプリ開発ではたいてい必要ありません。

使用例：

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

システムイベントのバイナリログを監視します。このバッファには普通のテキストではなく、Log.event の呼び出しが残した短いエンコード済みの記録が入っています。am_proc_start や am_activity_launch などのタグで表示されます。パフォーマンス分析に使われます。

使用例：

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

カーネルのログを監視します。Linux カーネルが書いたメッセージを表示し、通常は dmesg コマンドの出力に似ています。ドライバのエラー、メモリの問題、温度や電源管理の情報がここにあります。root が必要な場合があります。

使用例：

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

すべてのバッファを一度に監視します。main、system、crash、radio、events、kernel が混ざって流れます。すべてを見たいときに便利ですが、出力がとても多くなるため、通常はフィルタと組み合わせます。

使用例：

```
adb logcat -b all
```

使用例（フィルタ付き）：

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B 複数のバッファ

カンマで区切った複数のバッファを同時に監視します。たとえばアプリのログとクラッシュのログを一緒に見られます。必要なチャンネルだけを選べば出力を減らせます。

使用例：

```
adb logcat -b main,crash
```

使用例（3 つのチャンネル）：

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

デフォルトの短い出力形式です。各行は優先度とタグだけで始まります。ざっと見るには良いですが、タイムスタンプがないためイベントの順序をつかみにくくなります。

使用例：

```
adb logcat -v brief
```

出力例：

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

日付と時刻を追加します。各イベントがいつ起きたかがわかるので、短い形式よりずっと便利です。秒の後にミリ秒も表示されます。

使用例：

```
adb logcat -v time
```

出力例：

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

最もおすすめの出力形式です。日付、時刻、優先度、タグ、PID（プロセス ID）、TID（スレッド ID）を含みます。マルチスレッドのアプリでは、どのスレッドが何をしたかを知るのに欠かせません。Android Studio がデフォルトで使う形式でもあります。

使用例：

```
adb logcat -v threadtime
```

出力例：

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

すべてのメタデータ項目を含む詳細な出力で、エントリの間に空行が入ります。長いセッションでは各ログをひとつのブロックとして表示しますが、場所を取ります。

使用例：

```
adb logcat -v long
```

出力例：

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

優先度に応じてログに色を付けます。エラーは赤、警告はオレンジ、情報は緑、デバッグは青です。長い流れの中でも目がすぐエラーに向きます。ターミナルが色に対応している必要があります。

使用例：

```
adb logcat -v color
```

使用例（time と一緒に）：

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

時刻の形式に年を追加します。1 年以上にわたる長いログやアーカイブで使います。通常の開発ではたいてい必要ありません。

使用例：

```
adb logcat -v year
```

出力例：

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

ログを出したアプリのユーザー ID（UID）を表示します。複数のアプリが同じタグを使うときに役立ちます。修飾子なので、他の形式と組み合わせて使います。

使用例：

```
adb logcat -v uid
```

出力例：

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat タグと優先度のフィルタ

特定のタグのログを、特定の優先度以上で表示します。優先度は V（Verbose）が最も低く、D（Debug）、I（Info）、W（Warning）、E（Error）、F（Fatal）と続き、S（Silent）が最も高くなります。このフィルタ方法は logcat の最も強力な機能です。

使用例（エラーのみ）：

```
adb logcat *:E
```

使用例（警告以上）：

```
adb logcat *:W
```

使用例（許可リスト）：

```
adb logcat ActivityManager:I MyApp:D *:S
```

最後の例は ActivityManager を info 以上、MyApp を debug 以上で表示し、それ以外のタグはすべて非表示にします。一部のシェルでは * が特殊文字なので、引用符で囲む必要があります：

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S タグ

サイレントフィルタの省略形です。指定したタグだけを表示し、それ以外はすべて非表示にします。開発中に自分のタグに集中する最も手軽な方法です。

使用例：

```
adb logcat -s MyApp
```

使用例（複数のタグ）：

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

上の例では 3 つのタグのログだけが表示され、ほかはすべて非表示になります。

---

24- Adb Logcat -S と優先度

タグと優先度を一緒に指定するのに使います。たとえば MyApp タグの警告以上だけを見るときです。

使用例：

```
adb logcat MyApp:W *:S
```

このコマンドは MyApp タグの warning、error、fatal のログだけを表示します。

---

25- Adb Logcat --Pid フィルタ

特定のプロセス ID（PID）のログだけを表示します。アプリの PID を知るには、まず adb shell pidof を使い、その PID で絞り込みます。1 つのセッションのログをすべて切り分けるのに最適です。

使用例（PID を取得）：

```
adb shell pidof com.example.app
```

出力例：

```
5678
```

使用例（絞り込み）：

```
adb logcat --pid=5678
```

使用例（1 行で）：

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T 時刻

指定したタイムスタンプ以降のログを表示します。つまり「この瞬間より後のものを持ってくる」です。大文字の T で書きます。小文字の t は別の意味です。

使用例：

```
adb logcat -T "09-21 14:00:00.000"
```

使用例（今日から）：

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T 行数

小文字の t で、指定した数の最新の行を表示してコマンドが終了します。-d と同じように動きますが、最後の N 行だけを出します。ざっと見るのに最適です。

使用例（最後の 100 行）：

```
adb logcat -t 100
```

使用例（最後の 500 行とフィルタ）：

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

正規表現（regex）で内容を検索し、一致する行を表示します。テキストの素早い検索にとても強力で、特に複数の単語を一度に探すときはフィルタより速くなります。

使用例（1 つの単語）：

```
adb logcat -e "Login"
```

使用例（複数の単語）：

```
adb logcat -e "Login|Error|Crash"
```

使用例（パッケージ名付き）：

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B バイナリ出力

ログをバイナリ形式で書き出します。人は読めませんが、別の logcat で読み直したり、他のツールで処理したりできます。自動解析システムで使われます。

使用例：

```
adb logcat -B > Ham_Log.Bin
```

あとでそのファイルは次のように読みます：

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S 統計

ログの統計を表示します。各タグがどれだけのログを出したか、どのタグが「chatty」警告を引き起こしたかがわかります。あるアプリがログをあふれさせていると思ったら、このコマンドで確かめられます。

使用例：

```
adb logcat -S
```

出力例：

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R と -N の併用

ローテーションするログ用です。-r は各ファイルのサイズを、-n は保持するファイル数を決めます。アプリを長時間動かして、継続的にログを取りたいときに便利です。古いファイルは削除され、新しいファイルが開かれます。

使用例：

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

上の例では、ファイルが 4 MiB に達するたびに新しいファイルが開かれ、合計 10 個のファイルが保持されます。こうしてディスクが際限なく埋まることはありません。

---

32- Adb Logcat -P フィルタスクリプト

logcat に独自のフィルタスクリプトを指定します。Android 独自のフィルタ形式で、通常はシステム開発者が使います。アプリ開発者が使うことはまれです。

使用例：

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L タイムスタンプ

一部の形式で、各行の先頭にタイムスタンプを追加するオプションです。動作は Android のバージョンによって異なります。たいていは -v time と同じことを別の方法で行います。

使用例：

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

各行を printf 形式、つまり 優先度 (PID) タグ: メッセージ で表示します。古いツールとの互換性のためのもので、最近の Android ではほとんど必要ありません。

使用例：

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

メタデータなしの生のメッセージだけを表示します。タグも優先度もタイムスタンプもなく、ログ本文だけが書かれます。出力を別のプログラムに渡すのに便利です。

使用例：

```
adb logcat -v raw
```

出力例：

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

各行の先頭にタグだけを書き、その後に残りを書きます。優先度と PID は隠れます。1 つのタグに集中しているときに便利です。

使用例：

```
adb logcat -v tag
```

出力例：

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

各行の先頭にまず PID、次にタグを書きます。プロセスごとの分析に便利で、どのプロセスが何を書いたかがすぐにわかります。

使用例：

```
adb logcat -v process
```

出力例：

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

各行の先頭にスレッド ID（TID）だけを書き、その後に残りを書きます。マルチスレッドの分析で、どのスレッドが動いているかを示します。短い形式です。

使用例：

```
adb logcat -v thread
```

出力例：

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D と -T の併用

ある時間範囲のログ、たとえば直近 1 時間のログを取得してファイルに保存します。

使用例：

```
adb logcat -d -T "09-21 13:00:00.000"
```

使用例（ファイルへ）：

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C と -D を順に使う

まず消去し、その後で収集する流れです。たとえばテストの開始前に消去し、テストの終わりにログを収集します。

使用例：

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat をファイルに保存してパソコンへ移す

スマホのターミナルで長時間記録するのに便利です。まずデバイス上のファイルに書き込み、その後 adb pull でパソコンに移します。

使用例（デバイスに保存）：

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

使用例（パソコンへ移す）：

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat と Grep

スマホのターミナル（Termux）で grep を使ってテキストを素早く検索します。grep のすべてのオプションが使えるため、regex フィルタより柔軟な場合があります。

使用例：

```
adb logcat | grep "Hata"
```

使用例（大文字と小文字を区別しない）：

```
adb logcat | grep -I "error"
```

使用例（行番号付き）：

```
adb logcat | grep -N "Exception"
```

使用例（複数の単語）：

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat と Tail

ライブの流れを最後の数行に絞ります。ログ全体ではなく終わりだけを見たいときに使います。

使用例：

```
adb logcat | tail -100
```

使用例（継続して追いかける）：

```
adb logcat | tail -F
```

---

44- Adb Logcat と Head

最初の数行を表示します。長いログをざっと見るのに向いています。

使用例：

```
adb logcat -d | head -50
```

---

45- Adb Logcat と Wc

総行数を表示します。たとえばテストがどれだけのログを出したかを測るのに使います。

使用例：

```
adb logcat -d | wc -l
```

使用例（エラーだけを数える）：

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat と Sort

ログをアルファベット順や数値順に並べ替えます。主に分析のときに使います。

使用例：

```
adb logcat -d | sort
```

使用例（逆順）：

```
adb logcat -d | sort -R
```

---

47- Adb Logcat と Uniq

重複した行を取り除きます。アプリが同じエラーを何度も書くとき、このコマンドで出力が簡潔になります。

使用例：

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

上の例は、最も多く繰り返された 20 行を数えて表示します。デバッグでとても役立ちます。

---

48- Adb Logcat と Awk

特定の列を取り出したり、テキストを加工したりします。たとえば threadtime 形式の PID 列です。

使用例（PID 列だけ）：

```
adb logcat -v threadtime | awk '{print $3}'
```

使用例（タグとメッセージだけ）：

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat と Xargs

各行ごとにコマンドを実行します。たとえばエラーの行ごとに通知を送ります。主に自動化で使われます。

使用例：

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat と Sed

テキストを置き換えます。出力の特定のパターンを変えるのに最適です。

使用例：

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

上の例は、すべての MyApp を Uygulama に置き換えて表示します。

---

51- Adb Logcat と Tee

画面に表示しながら、同時にファイルにも書き込みます。ライブでの観察と記録が一緒に行われます。

使用例：

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat と Watch

一定の間隔でログを取得します。たとえば 5 秒ごとの要約です。

使用例：

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat と Timeout

決めた時間が過ぎたら logcat を止めます。自動テストでとても役立ちます。

使用例：

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat と Alias

よく使うコマンドを短くします。Termux では .bashrc または .zshrc ファイルに追加します。

使用例（alias を定義）：

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

あとは次のように呼ぶだけです：

```
lce
```

---

55- Adb Logcat と複数のデバイス

複数のデバイスが接続されているとき、特定のデバイスを指定します。まず adb devices でデバイスを一覧にし、-s で 1 台を選びます。

使用例：

```
adb devices
```

出力例：

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

使用例（特定のデバイス）：

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat とワイヤレスデバッグ

ワイヤレスデバッグ用です。デバイスが同じ Wi-Fi ネットワークにあれば、ケーブルなしで接続できます。Android 11 以降で対応しています。

使用例（デバイスでワイヤレスデバッグをオンにしてから）：

```
adb pair 192.168.1.42:37123
```

使用例（接続）：

```
adb connect 192.168.1.42:5555
```

使用例（logcat）：

```
adb logcat -v threadtime
```

---

57- Adb Logcat と Bugreport

システム全体の詳細なレポートを作成します。logcat に加えて dumpsys、dmesg、top など多くのツールが実行され、すべてが 1 つの zip にまとめられます。最も包括的なデバッグ方法です。

使用例：

```
adb bugreport /Sdcard/Bugreport.Zip
```

使用例（パソコンへ移す）：

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat と Dumpsys

logcat と並べて、システムサービスの現在の状態を表示します。たとえば ActivityManager がその瞬間に何をしているかです。

使用例：

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat と ANR トレース

アプリが応答しない（ANR）ときのスタックトレースを取得します。ANR の記録は /Data/Anr/Traces.Txt ファイルにあります。

使用例：

```
adb shell cat /Data/Anr/Traces.Txt
```

使用例（root が必要）：

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat と Tombstone

ネイティブクラッシュ用です。/Data/Tombstones/ フォルダにクラッシュごとにファイルが作られます。C++ や Rust のコードにエラーがあるときはここを見ます。

使用例：

```
adb shell ls /Data/Tombstones/
```

使用例（最新の tombstone）：

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat と Systrace

システム全体のパフォーマンストレース用です。ログだけでなく、カーネルレベルのトレースデータを収集します。Perfetto と一緒に使います。

使用例：

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat と Layout Inspector

画面上のアプリのレイアウトを分析します。Layout Inspector と一緒に使いますが、関連するログは logcat でも追えます。

使用例：

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat と Memory Profiler

メモリの使用状況を監視します。Memory Profiler は通常 Android Studio にありますが、メモリ関連のログは logcat でも見られます。

使用例：

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat と CPU Profiler

CPU の使用状況を監視します。簡単な測定なら top コマンドでも十分です。

使用例：

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat と Network Profiler

ネットワークの通信量を監視します。OkHttp や Retrofit などのライブラリは logcat に書き込みますが、完全な分析には Network Profiler が必要です。

使用例：

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat と GPU Profiler

GPU の使用状況を監視します。簡単な測定なら gfxinfo コマンドでも十分です。

使用例：

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat と Battery Historian

バッテリーの消費を監視します。dumpsys battery でも確認できます。

使用例：

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat で Wi-Fi を診断

接続の切断や電波の弱さなど、Wi-Fi の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat で Bluetooth を診断

ペアリングの失敗や接続の切断など、Bluetooth の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat でセンサーを診断

加速度計、ジャイロスコープ、磁力計などのセンサーの問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat でカメラを診断

写真の撮影や動画の録画など、カメラの問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat で音声を診断

音が出ない、マイクが動かないなど、音声の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat で位置情報を診断

GPS 信号がない、位置が更新されないなど、位置情報の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat で通知を診断

通知が表示されない、間違ったタイミングで届くなど、通知の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat で権限を診断

権限が付与されない、拒否されるなど、権限の問題をデバッグするのに使います。

使用例：

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat で WebView を診断

ページが読み込まれない、JavaScript エラーが出るなど、WebView の問題をデバッグするのに使います。

使用例：

```
adb logcat -s Chromium WebView
```

使用例（より詳しく）：

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat で OkHttp を診断

OkHttp の問題、つまりネットワークリクエスト、レスポンス、エラーをデバッグするのに使います。

使用例：

```
adb logcat -s Okhttp
```

使用例（すべてのレベル）：

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat で Retrofit を診断

Retrofit の問題をデバッグするのに使います。通常 OkHttp の上で動くため、OkHttp のログを見ます。

使用例：

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat で Glide を診断

画像が読み込まれない、メモリの問題など、Glide（画像読み込みライブラリ）の問題をデバッグするのに使います。

使用例：

```
adb logcat -s Glide
```

---

80- Adb Logcat で Picasso を診断

Picasso（画像読み込みライブラリ）の問題をデバッグするのに使います。

使用例：

```
adb logcat -s Picasso
```

---

81- Adb Logcat で Room を診断

Room（データベースライブラリ）の問題、つまりクエリのエラーやスキーマの問題をデバッグするのに使います。

使用例：

```
adb logcat -s Room
```

---

82- Adb Logcat で Firebase を診断

Crashlytics、Analytics、Messaging などのモジュールで Firebase の問題をデバッグするのに使います。

使用例：

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat で Crashlytics を診断

Firebase Crashlytics の問題をデバッグするのに使います。

使用例：

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat で Analytics を診断

分析の問題、つまり送信されないイベントや一括送信の問題をデバッグするのに使います。

使用例：

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat と ADB ワイヤレスデバッグ

ワイヤレスデバッグでは adb の再起動が必要になることがあります。まず USB で接続し、その後ワイヤレスに切り替えます。

使用例：

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat とデバイスの再起動

デバイスを再起動してログを監視します。特に起動時に起きるエラーを見るためです。

使用例（ターミナル 2 つ）：

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat とアプリの再起動

1 つのアプリを再起動してログを監視します。

使用例：

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat とアプリのインストール

APK をインストールしてログを監視します。

使用例：

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat とアプリのアンインストール

アプリをアンインストールしてログを監視します。

使用例：

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat とスクリーンショット

スクリーンショットを撮り、エラーの瞬間のログを保存します。

使用例：

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat と画面録画

画面を録画し、エラーの瞬間のログを保存します。Android 4.4 以降で対応しています。

使用例：

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat と Monkey テスト

ランダムなユーザー操作をシミュレートしてアプリをテストします。ログと組み合わせるととても効果的です。

使用例：

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat と UI Automator テスト

UI 自動化テスト用で、テスト中にログを監視します。

使用例：

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat と Espresso テスト

UI 自動化の一部である Espresso テスト用です。

使用例：

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat と JUnit テスト

ユニットテスト用です。通常はパソコンで実行しますが、デバイス上でも実行できます。

使用例：

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat と StrictMode

アプリのパフォーマンスの問題を見つけます。メインスレッドでディスクやネットワークにアクセスすると警告します。

使用例：

```
adb logcat -s StrictMode
```

---

97- Adb Logcat と Choreographer

スキップされたフレームを見つけます。アニメーションがカクつくときはここを見ます。

使用例：

```
adb logcat -s Choreographer
```

---

98- Adb Logcat と GPU のオーバードロー

オーバードローの問題を見つけます。グラフィック性能の分析で使います。

使用例：

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat とレンダースレッド

レンダースレッドの問題を見つけます。最近の Android では描画はこのスレッドで行われます。

使用例：

```
adb logcat -s RenderThread
```

---

100- Adb Logcat と GPU レンダリング

GPU レンダリングの問題を見つけます。GPU レンダリングのプロファイルをオンにした後にここを見ます。

使用例：

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat と TextView

フォントの問題や測定のエラーなど、テキストビューの問題を見つけます。

使用例：

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat と RecyclerView

リストビューの問題、つまりスクロール性能とビューの再利用の問題を見つけます。

使用例：

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat と ViewPager

ページ切り替えの問題を見つけます。

使用例：

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat と Fragment

Fragment のライフサイクルの問題を見つけます。

使用例：

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat と Activity

Activity のライフサイクルの問題を見つけます。

使用例：

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat とサービス

バックグラウンドサービスの問題を見つけます。

使用例：

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat と WorkManager

バックグラウンドタスク用の WorkManager の問題を見つけます。

使用例：

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat と AlarmManager

スケジュールされたタスク用の AlarmManager の問題を見つけます。

使用例：

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat と SharedPreferences

SharedPreferences（シンプルなデータ保存）の問題を見つけます。

使用例：

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat と SQLite

SQLite（ローカルデータベース）の問題を見つけます。

使用例：

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat とファイル操作

ファイルの読み書きの問題を見つけます。

使用例：

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat とネットワーク操作

HTTP、TCP、UDP などのネットワーク接続の問題を見つけます。

使用例：

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat と SSL

SSL 証明書の問題を見つけます。安全な接続が失敗するときはここを見ます。

使用例：

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat と JSON

JSON の解析の問題を見つけます。

使用例：

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat と XML

XML の解析の問題を見つけます。

使用例：

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat と画像

画像の読み込みと処理の問題を見つけます。

使用例：

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat とメディア

メディアの再生と録画の問題を見つけます。

使用例：

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat と ExoPlayer

ExoPlayer（メディアプレーヤーライブラリ）の問題を見つけます。

使用例：

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat と Chromecast

Chromecast の問題を見つけます。

使用例：

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat と広告

広告ライブラリの問題を見つけます。

使用例：

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat と課金

Google Play の課金の問題を見つけます。

使用例：

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat と地図

地図ライブラリの問題を見つけます。

使用例：

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat と認証

Google Sign-In や Firebase Auth などのログインの問題を見つけます。

使用例：

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat と通知

通知の送信と受信の問題を見つけます。

使用例：

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat とディープリンク

URL からアプリを開くなど、ディープリンクの問題を見つけます。

使用例：

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat と Intent

アプリ間の移動での Intent の問題を見つけます。

使用例：

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat とブロードキャスト

システムとアプリの間のブロードキャストメッセージの問題を見つけます。

使用例：

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat とコンテンツプロバイダ

アプリ間でデータを共有するコンテンツプロバイダの問題を見つけます。

使用例：

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat と JobScheduler

スケジュールされたバックグラウンドタスク用の JobScheduler の問題を見つけます。

使用例：

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat と Doze モード

デバイスがスリープしている間のバックグラウンド処理に関わる Doze（スリープ）モードの問題を見つけます。

使用例：

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat とバッテリーの最適化

バックグラウンドの制限に関わるバッテリー最適化の問題を見つけます。

使用例：

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat とメモリリーク

メモリリークを見つけます。LeakCanary などのツールと一緒に使います。

使用例：

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat と ANR

ANR（アプリが応答しない）の問題を見つけます。

使用例：

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat とクラッシュ

クラッシュを見つけます。

使用例：

```
adb logcat -b crash
```

---

135- Adb Logcat とネイティブクラッシュ

ネイティブクラッシュ（C/C++/Rust）を見つけます。

使用例：

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat と Tombstone 分析

ネイティブクラッシュの記録を分析します。

使用例：

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat と Systrace 分析

システムトレースの記録を分析します。

使用例：

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat と Perfetto 分析

最新のシステムトレースツールで、logcat より包括的です。

使用例：

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat と Atrace 分析

システムレベルのトレース用で、Perfetto の古い形です。

使用例：

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat と Traceview 分析

メソッドレベルのプロファイリング用です。

使用例：

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat とメソッドトレース

特定のメソッドを追跡します。

使用例：

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat とヒープダンプ

メモリのダンプを作成します。メモリ分析でとても役立ちます。

使用例：

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat と GC ログ

ガベージコレクションのイベントを監視します。

使用例：

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat と JNI デバッグ

JNI（Java Native Interface）の問題を見つけます。

使用例：

```
adb logcat -s JNI
```

---

145- Adb Logcat と ART デバッグ

ART（Android Runtime）の問題を見つけます。

使用例：

```
adb logcat -s Art
```

---

146- Adb Logcat と Dalvik デバッグ

Dalvik 仮想マシンの問題を見つけます。古い Android バージョンで使います。

使用例：

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat と Chatty の問題

logcat がログを書きすぎると、「chatty」警告を出して行を飛ばします。フィルタか、より大きなバッファで解決します。

使用例（バッファを大きくする）：

```
adb logcat -G 16M
```

使用例（フィルタを使う）：

```
adb logcat -s MyApp
```

---

148- Adb Logcat とカラー出力

長いログの流れでは、カラー出力は目が疲れにくく、エラーがすぐに見つかります。

使用例：

```
adb logcat -v color
```

使用例（time と一緒に）：

```
adb logcat -v time,color
```

---

149- Adb Logcat と Bash スクリプト

ログを自動で集めるスクリプトを書くのに使います。その上にバグ報告の仕組みを作れます。

スクリプト例：

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat のまとめ

この一覧が示すとおり、logcat は 1 つのコマンドではなく、コマンドの集まりです。フィルタ、形式、バッファ、ファイル、その他のツールと組み合わせることで、デバッグの最強の武器になります。どの状況でどの組み合わせを使うかがわかっていれば、バグを見つけるのに数分しかかかりません。覚えておいてください。デバッグセッションの最初のルールはいつも同じです。まず adb logcat -c で消去し、次にバグを再現し、それからフィルタした logcat で観察します。

@@ Android Debug Bridge 接続
@: ADB ワイヤレス接続コマンド
1- ワイヤレスデバッグを有効にする

まずデバイスで開発者向けオプションを有効にする必要があります。そのために 設定 → デバイス情報 → ビルド番号 を 7 回タップします。次に 設定 → システム → 開発者向けオプション → ワイヤレスデバッグ をオンにします。この手順を行わないと、adb pair も adb connect も動きません。

パスの例：

```
設定 → デバイス情報 → ビルド番号（7 回タップ）
設定 → システム → 開発者向けオプション → ワイヤレスデバッグ（オン）
```

---

2- デバイスの IP アドレスとポートを調べる

ワイヤレスデバッグがオンの状態で「ペア設定コードによるデバイスのペア設定」を開きます。そこにデバイスの IP アドレスとランダムなポート番号が表示されます。たとえば 192.168.1.42:37123 です。このポートはペア設定専用で、その後は変わります。同じ画面に 6 桁のペア設定コードも表示されます。

画面の例：

```
IP アドレスとポート
192.168.1.42:37123

Wi-Fi ペア設定コード
482915
```

---

3- Adb Pair コマンド

このコマンドでデバイスとペア設定します。パソコンまたは Termux で実行します。デバイスに表示された IP アドレスとペア設定用のポートを入力します。実行すると、ターミナルが Enter pairing code: と尋ねて 6 桁のコードを待ちます。コードが正しければ Successfully paired というメッセージが表示されます。

使用例：

```
adb pair 192.168.1.42:37123
```

出力例：

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

コードが間違っていると、次のように失敗します：

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Adb Pair にコードを渡すほかの方法

毎回手でコードを入力する必要はありません。バージョンによってはパラメータとして直接受け付けますが、セキュリティ上の理由から通常は対応していません。それでも試すことはできます。

使用例（1 行で）：

```
adb pair 192.168.1.42:37123 482915
```

使用例（パイプで）：

```
echo "482915" | adb pair 192.168.1.42:37123
```

これらの方法はバージョンによって変わります。最も確実なのは、コマンドを実行して手でコードを入力することです。

---

5- Adb Pair で注意すること

ペア設定用のポートと接続用のポートは別物です。ペア設定用のポートは通常ランダムで大きな番号、接続用のポートは通常 5555 です。ペア設定は 1 回だけで、同じデバイスに再接続するには adb connect で十分です。ただしデバイスを出荷時の状態にリセットしたり、ワイヤレスデバッグをオフにしたりするとペア設定は消え、もう一度 adb pair が必要になります。

比較の例：

```
ペア設定：adb pair 192.168.1.42:37123   → コードを求める、1 回だけ
接続：    adb connect 192.168.1.42:5555  → コード不要、毎回
```

---

6- Adb Connect コマンド

このコマンドはペア設定済みのデバイスに接続します。ワイヤレスデバッグのメイン画面に表示された IP アドレスとポートを入力します。ポートは通常 5555 ですが、異なることもあります。接続に成功すると Connected to というメッセージが表示されます。

使用例：

```
adb connect 192.168.1.42:5555
```

出力例：

```
Connected to 192.168.1.42:5555
```

すでに接続されている場合は、次のように知らせます：

```
Already connected to 192.168.1.42:5555
```

---

7- ポートなしの Adb Connect

一部のバージョンでは、ポートを指定しないとデフォルトで 5555 が試されます。ただしすべてのデバイスで同じようには動かないため、ポートを明示的に書くのが最も確実です。

使用例（デフォルトのポート）：

```
adb connect 192.168.1.42
```

使用例（明示的なポート）：

```
adb connect 192.168.1.42:5555
```

---

8- 接続を確認する

接続した後、adb devices コマンドでデバイスが本当に接続されているかがわかります。デバイスの IP アドレスが一覧に表示され、横に device と書かれている必要があります。offline と書かれていれば、接続はできたもののまだ安定していません。少し待ってからもう一度試してください。

使用例：

```
adb devices
```

出力例：

```
List of devices attached
192.168.1.42:5555   device
```

出力例（未接続）：

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Adb Disconnect コマンド

接続を切ります。デバイスが同じネットワークにいても adb の接続は終了します。再接続するには adb connect をもう一度実行するだけで、ペア設定は不要です。

使用例（1 台のデバイス）：

```
adb disconnect 192.168.1.42:5555
```

出力例：

```
Disconnected 192.168.1.42:5555
```

使用例（すべてのデバイス）：

```
adb disconnect
```

出力例：

```
Disconnected everything
```

---

10- Adb Tcpip コマンド

USB で接続されたデバイスをワイヤレスモードに切り替えます。このコマンドはデバイスに指定したポート（通常は 5555）で待ち受けさせます。adb pair のない Android 11 より前のバージョンでよく使われる方法です。

使用例：

```
adb tcpip 5555
```

出力例：

```
Restarting in TCP mode port: 5555
```

その後 USB ケーブルを外し、次のコマンドを実行します：

```
adb connect 192.168.1.42:5555
```

---

11- Adb Usb コマンド

ワイヤレスモードから USB モードに戻します。デバイスをもう一度ケーブルでつなぎたいときに使います。

使用例：

```
adb usb
```

出力例：

```
Restarting in USB mode
```

---

12- Adb Kill-Server と Start-Server

adb サーバーが固まったり、接続を拒否したりすることがあります。そのときはサーバーを止めて、もう一度起動する必要があります。これですべての接続がリセットされ、きれいな状態から始められます。

使用例：

```
adb kill-server
adb start-server
```

出力例：

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

その後、もう一度接続します：

```
adb connect 192.168.1.42:5555
```

---

13- 同じネットワークにいること

ワイヤレス adb では、デバイスとパソコンまたはスマホが同じ Wi-Fi ネットワークにいる必要があります。異なるネットワークでは接続できません。ゲストネットワーク、5 GHz と 2.4 GHz の分離、VPN やファイアウォールが問題を起こすことがあります。簡単な解決策は、両方の機器を同じルーターにつなぎ、VPN をオフにすることです。

確認の例：

```
デバイス：設定 → Wi-Fi → 接続中のネットワーク（例：Home_Wifi）
Termux：設定 → Wi-Fi → 接続中のネットワーク（例：Home_Wifi）
```

---

14- IP アドレスを手で調べる

デバイスの IP アドレスを調べるには 設定 → Wi-Fi → 接続中のネットワーク → ネットワークの詳細 を開きます。または下のコマンドで Termux から確認します。ただし IP アドレスは時間とともに変わることがあるため、毎回ワイヤレスデバッグの画面を確認するのが最も確実です。

使用例（Termux）：

```
ifconfig wlan0
```

使用例（デバイス上）：

```
設定 → Wi-Fi → 接続中のネットワーク → IP アドレス
```

---

15- 同じデバイスの Termux から自分自身に接続する

Termux が同じデバイスで動いているなら adb はすでにローカルですが、それでも adb connect で localhost 経由で接続できます。通常は 127.0.0.1:5555 を使います。拒否するデバイスもあり、その場合はデバイス自身のローカル IP を使います。

使用例（localhost）：

```
adb connect 127.0.0.1:5555
```

使用例（自身のローカル IP）：

```
adb connect 192.168.1.42:5555
```

---

16- 複数のデバイスが接続されているときのコマンド実行

複数のデバイスが同時に接続されている場合は、各コマンドで -s パラメータを使ってデバイスを選びます。そうしないと adb は error: more than one device で失敗します。

使用例：

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- 接続が切れたとき

Wi-Fi の電波が弱くなったり、デバイスがスリープしたり、ネットワークが変わったりすると、接続は切れます。するとデバイスは adb devices の一覧から消えるか、offline と表示されます。解決策は adb connect をもう一度実行することです。ペア設定は通常有効なままで、adb pair は不要です。

復旧の例：

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- デバイスを再起動した後

デバイスを再起動すると、ワイヤレスデバッグは通常オフになります。オンのままのデバイスもありますが、ペア設定は消えます。デバイスが起動したらワイヤレスデバッグの画面を開き、新しいポートと新しいコードを確認して、もう一度 adb pair、次に adb connect を実行します。

流れの例：

```
デバイスを再起動 → ワイヤレスデバッグをオン
新しいポート：42157、新しいコード：735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- ペア設定でよくあるエラー

Failed: Wrong password → コードの入力が間違っています。
Failed to connect: Connection refused → ポートが間違っているか、デバイスが待ち受けていません。
Failed: Pairing rejected → デバイスがペア設定を拒否しました。コードを更新してください。
Timeout → ネットワークに問題があります。同じネットワークにいるか確認してください。

エラーと解決の例：

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

解決策：デバイスのコードが変わりました。新しいコードを確認して、もう一度試してください。

---

20- ワイヤレス Adb をオフにする

作業が終わったらワイヤレスデバッグをオフにするのが、セキュリティのために良い習慣です。設定からオフにするか、adb で USB モードに切り替えられます。

使用例：

```
adb usb
adb disconnect
```

使用例（設定から）：

```
設定 → システム → 開発者向けオプション → ワイヤレスデバッグ（オフ）
```

---

21- Android のバージョンによる違い

Android 11 以降：adb pair によるペア設定が必須で、ランダムなポートを使います。
Android 10 以前：adb tcpip 5555 でワイヤレスモードに切り替えます。ペア設定はありません。

比較の例：

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- セキュリティについて

ワイヤレスデバッグがオンの間は、同じネットワーク上の悪意のある人がデバイスへのアクセスを試みるかもしれません。そのため、信頼できない Wi-Fi ネットワークではこの機能をオフにしておきましょう。ペア設定コードは安全のためにありますが、それでも注意は必要です。作業が終わったらオフにしてください。

対策の例：

```
使わないとき：設定 → 開発者向けオプション → ワイヤレスデバッグ（オフ）
安全なネットワークで：自宅やオフィスでだけオンにする
```

---

23- すべての流れを 1 か所で

下のスクリプトは、ゼロから接続までのすべての手順を示しています。IP とポートを自分のデバイスのものに変えるだけです。

流れの例：

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

期待される結果：

```
List of devices attached
192.168.1.42:5555   device
```

---

24- 継続的な接続のためのショートカットスクリプト

頻繁に接続するなら、スクリプトを書いて毎回 1 つのコマンドで接続できます。たとえば Termux では ~/.bashrc ファイルに関数を追加できます。

関数の例：

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "デバイスに表示された IP:ポート とコードを入力してください："
    read -p "アドレス: " ADRES
    adb pair "$ADRES"
    read -p "接続ポート（例：5555）: " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

使い方：

```
wifi_adb
```

---

25- おわりに

ワイヤレス adb を使えば USB ケーブルなしでデバッグでき、スマホ自体の Termux で作業するときに特に便利です。基本の 2 つのコマンドは adb pair と adb connect です。前者は 1 回だけペア設定し、後者は接続のたびに使います。残りのコマンド（adb devices、adb disconnect、adb tcpip、adb kill-server）は、この 2 つを支え、問題を解決するためのものです。同じネットワークのルールを忘れず、使わないときはオフにし、何かおかしければサーバーを再起動しましょう。以上です。
"#;

const KOREAN: &str = r#"@@ Logcat 명령어
1- Adb Logcat

기기의 모든 로그를 실시간으로 터미널에 출력합니다. 명령을 실행한 순간부터 시스템, 앱, 서비스, 커널이 만드는 모든 로그 줄이 화면에 나타납니다. 멈추려면 키보드에서 Ctrl + C를 누르세요. 이 명령은 디버깅의 기본이지만, 필터 없이 쓰면 초당 수백 줄이 지나갈 수 있어 필요한 내용을 찾기 어려워집니다.

사용 예:

```
adb logcat
```

출력 예:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

지금까지 쌓인 로그를 보여 주고 명령이 끝납니다. 계속 이어지는 흐름 대신 그 순간의 스냅숏을 찍습니다. 버그 보고서를 준비할 때, 비정상 종료 후 상태를 저장할 때, 스크립트에서 로그를 한 번만 가져오고 싶을 때 사용하세요.

사용 예:

```
adb logcat -d
```

사용 예 (파일로 저장):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

이 명령은 보고서를 자동으로 서버에 보내는 스크립트에서 자주 쓰입니다. 스스로 끝나기 때문에 스크립트를 절대 멈춰 세우지 않습니다.

---

3- Adb Logcat -C

모든 로그 버퍼를 비우고 그 시점까지 쌓인 내용을 전부 지웁니다. 새 디버깅 세션 전에 실행하면 그 뒤에 생긴 로그만 보이고 예전 잡음에 헷갈리지 않습니다. 버그를 재현하려 할 때 보통 첫 단계입니다.

사용 예:

```
adb logcat -c
adb logcat
```

위의 두 명령을 차례로 실행하면 완전히 깨끗한 로그 흐름을 얻습니다. 비운 뒤에 일어난 일만 나타납니다.

---

4- Adb Logcat -G

현재 로그 버퍼의 크기와 얼마나 찼는지를 보여 줍니다. 각 버퍼의 사용량, 전체 용량, 사용된 양을 한 줄씩 나열합니다. 로그가 너무 빨리 흘러가 오래된 항목이 버려진다면(이를 “chatty”라고 합니다) 먼저 여기를 보세요.

사용 예:

```
adb logcat -g
```

출력 예:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G 크기

로그 버퍼의 크기를 바꿉니다. 예를 들어 256 KiB 버퍼를 16 MiB로 늘릴 수 있습니다. 그러면 더 많은 로그가 보관되고 오래된 항목이 더 늦게 버려집니다. 긴 테스트나 로그가 아주 많은 앱을 디버깅할 때 매우 유용합니다. 설정은 기기를 다시 시작하면 원래대로 돌아갑니다.

사용 예:

```
adb logcat -G 16M
```

사용 예 (줄이기):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F 파일

로그를 터미널에 보여 주는 대신 지정한 파일에 씁니다. 터미널은 깔끔하게 유지되고 로그는 보존됩니다. 파일 경로는 컴퓨터가 아니라 기기에 있습니다. 모바일 터미널(Termux)에서는 보통 /Sdcard/에 씁니다.

사용 예:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

사용 예 (파일에 이어 쓰기):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

위 예에서는 파일이 4 MiB에 이르면 새 파일이 열리고, 모두 10개의 파일이 보관됩니다.

---

7- Adb Logcat -B Main

앱의 메인 버퍼만 봅니다. 여기에는 앱이 Log.d, Log.i, Log.e 같은 호출로 쓴 로그가 들어 있습니다. 기본으로도 보이지만, 이 버퍼만 원할 때 명시적으로 지정합니다. 앱을 개발할 때 가장 자주 보는 곳입니다.

사용 예:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

시스템 서비스의 로그를 봅니다. 여기에는 ActivityManager, WindowManager, PackageManager, PowerManager 같은 서비스의 로그가 들어 있습니다. 앱 시작, 권한 부여, 서비스 연결 같은 이벤트를 보는 데 씁니다. 보통 main과 함께 봅니다.

사용 예:

```
adb logcat -b system
```

출력 예:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

비정상 종료 로그만 봅니다. 여기에는 앱이 강제로 닫힐 때 기록되는 FATAL EXCEPTION 블록이 들어 있습니다. 비정상 종료를 디버깅할 때 이 버퍼를 먼저 보는 것이 가장 빠른 길입니다. 스택 트레이스가 바로 여기에 있기 때문입니다.

사용 예:

```
adb logcat -b crash
```

출력 예:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

무선, 전화, 신호, 데이터 연결의 로그를 봅니다. SIM 카드, 네트워크 등록, 4G/5G 연결, SMS나 통화를 다룰 때 유용합니다. 일반적인 앱 개발에서는 보통 필요하지 않습니다.

사용 예:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

시스템 이벤트의 바이너리 로그를 봅니다. 이 버퍼에는 일반 텍스트가 아니라 Log.event 호출이 남긴 짧은 인코딩 기록이 들어 있습니다. am_proc_start, am_activity_launch 같은 태그로 나타납니다. 성능 분석에 쓰입니다.

사용 예:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

커널 로그를 봅니다. Linux 커널이 쓴 메시지를 보여 주며, 보통 dmesg 명령의 출력과 비슷합니다. 드라이버 오류, 메모리 문제, 온도와 전원 관리 정보가 여기에 있습니다. root가 필요할 수 있습니다.

사용 예:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

모든 버퍼를 한 번에 봅니다. main, system, crash, radio, events, kernel이 섞여서 흘러갑니다. 모든 것을 보고 싶을 때 유용하지만 출력이 매우 많아지므로 보통 필터와 함께 씁니다.

사용 예:

```
adb logcat -b all
```

사용 예 (필터와 함께):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B 여러 버퍼

쉼표로 구분한 여러 버퍼를 동시에 봅니다. 예를 들어 앱 로그와 비정상 종료 로그를 함께 볼 수 있습니다. 필요한 채널만 고르면 출력이 줄어듭니다.

사용 예:

```
adb logcat -b main,crash
```

사용 예 (세 채널):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

기본 짧은 출력 형식입니다. 각 줄은 우선순위와 태그로만 시작합니다. 빠르게 훑어보기에는 좋지만 타임스탬프가 없어 이벤트 순서를 파악하기 어렵습니다.

사용 예:

```
adb logcat -v brief
```

출력 예:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

날짜와 시간을 추가합니다. 각 이벤트가 언제 일어났는지 볼 수 있어 짧은 형식보다 훨씬 유용합니다. 초 뒤에 밀리초도 표시됩니다.

사용 예:

```
adb logcat -v time
```

출력 예:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

가장 권장되는 출력 형식입니다. 날짜, 시간, 우선순위, 태그, PID(프로세스 식별자), TID(스레드 식별자)를 포함합니다. 멀티스레드 앱에서는 어떤 스레드가 무엇을 했는지 알기 위해 꼭 필요합니다. Android Studio가 기본으로 쓰는 형식이기도 합니다.

사용 예:

```
adb logcat -v threadtime
```

출력 예:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

모든 메타데이터 필드를 담은 자세한 출력을 주며, 항목 사이에 빈 줄이 들어갑니다. 긴 세션에서 각 로그를 하나의 블록으로 보여 주지만 공간을 차지합니다.

사용 예:

```
adb logcat -v long
```

출력 예:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

우선순위에 따라 로그에 색을 입힙니다. 오류는 빨간색, 경고는 주황색, 정보는 초록색, 디버그는 파란색입니다. 긴 흐름 속에서도 눈이 바로 오류로 갑니다. 터미널이 색을 지원해야 합니다.

사용 예:

```
adb logcat -v color
```

사용 예 (time과 함께):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

시간 형식에 연도를 추가합니다. 1년 넘게 이어지는 긴 로그나 보관 자료에 씁니다. 일반적인 개발에서는 보통 필요하지 않습니다.

사용 예:

```
adb logcat -v year
```

출력 예:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

로그를 만든 앱의 사용자 식별자(UID)를 보여 줍니다. 여러 앱이 같은 태그를 쓸 때 유용합니다. 수정자이므로 다른 형식과 함께 씁니다.

사용 예:

```
adb logcat -v uid
```

출력 예:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat 태그와 우선순위 필터

특정 태그의 로그를 특정 우선순위부터 보여 줍니다. 우선순위는 V(Verbose)가 가장 낮고, D(Debug), I(Info), W(Warning), E(Error), F(Fatal) 순이며 S(Silent)가 가장 높습니다. 이 필터 방식이 logcat의 가장 강력한 기능입니다.

사용 예 (오류만):

```
adb logcat *:E
```

사용 예 (경고 이상):

```
adb logcat *:W
```

사용 예 (허용 목록):

```
adb logcat ActivityManager:I MyApp:D *:S
```

마지막 예는 ActivityManager를 info부터, MyApp을 debug부터 보여 주고 다른 모든 태그는 숨깁니다. 일부 셸에서는 * 문자가 특수 문자이므로 따옴표로 감싸야 합니다:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S 태그

조용한 필터의 줄임말입니다. 지정한 태그만 보여 주고 나머지는 모두 숨깁니다. 개발 중에 자신의 태그에 집중하는 가장 편한 방법입니다.

사용 예:

```
adb logcat -s MyApp
```

사용 예 (여러 태그):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

위 예에서는 세 태그의 로그만 통과하고 나머지는 모두 조용해집니다.

---

24- Adb Logcat -S 와 우선순위

태그와 우선순위를 함께 지정할 때 씁니다. 예를 들어 MyApp 태그의 경고 이상만 볼 때입니다.

사용 예:

```
adb logcat MyApp:W *:S
```

이 명령은 MyApp 태그의 warning, error, fatal 로그만 보여 줍니다.

---

25- Adb Logcat --Pid 필터

특정 프로세스 식별자(PID)의 로그만 보여 줍니다. 앱의 PID를 알려면 먼저 adb shell pidof를 쓰고, 그 PID로 걸러 냅니다. 한 세션의 로그를 모두 따로 떼어 내는 데 안성맞춤입니다.

사용 예 (PID 얻기):

```
adb shell pidof com.example.app
```

출력 예:

```
5678
```

사용 예 (필터):

```
adb logcat --pid=5678
```

사용 예 (한 줄로):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T 시간

지정한 타임스탬프부터의 로그를 보여 줍니다. 즉 “이 순간 이후의 것을 가져와”입니다. 대문자 T로 씁니다. 소문자 t는 다른 뜻입니다.

사용 예:

```
adb logcat -T "09-21 14:00:00.000"
```

사용 예 (오늘부터):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T 줄 수

소문자 t로 지정한 수만큼의 최신 줄을 보여 주고 명령이 끝납니다. -d처럼 동작하지만 마지막 N줄만 줍니다. 빠르게 훑어보기에 좋습니다.

사용 예 (마지막 100줄):

```
adb logcat -t 100
```

사용 예 (마지막 500줄과 필터):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

정규 표현식(regex)으로 내용을 검색하고 일치하는 줄을 보여 줍니다. 빠른 텍스트 검색에 아주 강력하며, 특히 여러 단어를 한 번에 찾을 때 필터보다 빠릅니다.

사용 예 (한 단어):

```
adb logcat -e "Login"
```

사용 예 (여러 단어):

```
adb logcat -e "Login|Error|Crash"
```

사용 예 (패키지 이름과 함께):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B 바이너리 출력

로그를 바이너리 형식으로 씁니다. 사람은 읽을 수 없지만 다른 logcat이 다시 읽거나 다른 도구가 처리할 수 있습니다. 자동 분석 시스템에서 쓰입니다.

사용 예:

```
adb logcat -B > Ham_Log.Bin
```

나중에 그 파일은 이렇게 읽습니다:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S 통계

로그 통계를 보여 줍니다. 각 태그가 로그를 얼마나 만들었는지, 어떤 태그가 “chatty” 경고를 일으켰는지 알 수 있습니다. 어떤 앱이 로그를 넘치게 쓴다고 생각되면 이 명령으로 확인할 수 있습니다.

사용 예:

```
adb logcat -S
```

출력 예:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R 과 -N 함께

순환 로그용입니다. -r은 각 파일의 크기를, -n은 보관할 파일 수를 정합니다. 앱이 오래 실행되고 계속 기록하고 싶을 때 유용합니다. 오래된 파일은 지워지고 새 파일이 열립니다.

사용 예:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

위 예에서는 파일이 4 MiB에 이를 때마다 새 파일이 열리고, 모두 10개의 파일이 보관됩니다. 그래서 디스크가 끝없이 차지 않습니다.

---

32- Adb Logcat -P 필터 스크립트

logcat에 사용자 지정 필터 스크립트를 지정합니다. Android 고유의 필터 형식으로 보통 시스템 개발자가 씁니다. 앱 개발자는 거의 쓰지 않습니다.

사용 예:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L 타임스탬프

일부 형식에서 각 줄 앞에 타임스탬프를 붙이는 옵션입니다. 동작은 Android 버전에 따라 다릅니다. 보통 -v time과 같은 일을 다른 방식으로 합니다.

사용 예:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

각 줄을 printf 형식, 즉 우선순위 (PID) 태그: 메시지로 보여 줍니다. 오래된 도구와의 호환을 위한 것이며, 최신 Android에서는 거의 필요하지 않습니다.

사용 예:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

메타데이터 없이 원시 메시지만 보여 줍니다. 태그, 우선순위, 타임스탬프 없이 로그 내용만 씁니다. 출력을 다른 프로그램에 넘길 때 유용합니다.

사용 예:

```
adb logcat -v raw
```

출력 예:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

각 줄 앞에 태그만 쓰고 그 뒤에 나머지를 씁니다. 우선순위와 PID는 숨겨집니다. 한 태그에 집중할 때 편리합니다.

사용 예:

```
adb logcat -v tag
```

출력 예:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

각 줄 앞에 먼저 PID, 그다음 태그를 씁니다. 프로세스별 분석에 편리하며, 어느 프로세스가 무엇을 썼는지 금방 보입니다.

사용 예:

```
adb logcat -v process
```

출력 예:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

각 줄 앞에 스레드 식별자(TID)만 쓰고 그 뒤에 나머지를 씁니다. 멀티스레드 분석에서 어느 스레드가 움직이는지 보여 줍니다. 짧은 형식입니다.

사용 예:

```
adb logcat -v thread
```

출력 예:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D 와 -T 함께

어떤 시간 구간의 로그, 예를 들어 최근 한 시간의 로그를 가져와 파일로 저장합니다.

사용 예:

```
adb logcat -d -T "09-21 13:00:00.000"
```

사용 예 (파일로):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C 와 -D 차례로

먼저 비우고 나중에 모으는 흐름입니다. 예를 들어 테스트를 시작하기 전에 비우고, 테스트가 끝날 때 로그를 모읍니다.

사용 예:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat 을 파일로 저장해 컴퓨터로 옮기기

모바일 터미널에서 오래 기록할 때 편리합니다. 먼저 기기의 파일에 쓰고, 그다음 adb pull로 컴퓨터에 옮깁니다.

사용 예 (기기에 저장):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

사용 예 (컴퓨터로 옮기기):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat 과 Grep

모바일 터미널(Termux)에서 grep으로 텍스트를 빠르게 찾을 때 씁니다. grep의 모든 옵션을 쓸 수 있어 regex 필터보다 유연할 수 있습니다.

사용 예:

```
adb logcat | grep "Hata"
```

사용 예 (대소문자 구분 없이):

```
adb logcat | grep -I "error"
```

사용 예 (줄 번호와 함께):

```
adb logcat | grep -N "Exception"
```

사용 예 (여러 단어):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat 과 Tail

실시간 흐름을 마지막 몇 줄로 줄입니다. 로그 전체가 아니라 끝부분만 보고 싶을 때 씁니다.

사용 예:

```
adb logcat | tail -100
```

사용 예 (계속 따라가기):

```
adb logcat | tail -F
```

---

44- Adb Logcat 과 Head

처음 몇 줄을 보여 줍니다. 긴 로그를 빠르게 훑어보기에 좋습니다.

사용 예:

```
adb logcat -d | head -50
```

---

45- Adb Logcat 과 Wc

전체 줄 수를 보여 줍니다. 예를 들어 테스트가 로그를 얼마나 만들었는지 잴 때 씁니다.

사용 예:

```
adb logcat -d | wc -l
```

사용 예 (오류만 세기):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat 과 Sort

로그를 알파벳순이나 숫자순으로 정렬합니다. 주로 분석할 때 씁니다.

사용 예:

```
adb logcat -d | sort
```

사용 예 (역순):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat 과 Uniq

반복되는 줄을 없앱니다. 앱이 같은 오류를 계속 쓰면 이 명령으로 출력이 간단해집니다.

사용 예:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

위 예는 가장 많이 반복된 20줄을 세어 보여 줍니다. 디버깅에 아주 유용합니다.

---

48- Adb Logcat 과 Awk

특정 열을 뽑아내거나 텍스트를 가공합니다. 예를 들어 threadtime 형식의 PID 열입니다.

사용 예 (PID 열만):

```
adb logcat -v threadtime | awk '{print $3}'
```

사용 예 (태그와 메시지만):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat 과 Xargs

각 줄마다 명령을 실행합니다. 예를 들어 오류 줄마다 알림을 보냅니다. 주로 자동화에서 씁니다.

사용 예:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat 과 Sed

텍스트를 바꿉니다. 출력의 특정 패턴을 바꾸기에 딱 좋습니다.

사용 예:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

위 예는 모든 MyApp 자리에 Uygulama를 씁니다.

---

51- Adb Logcat 과 Tee

화면에 보여 주면서 동시에 파일에도 씁니다. 실시간 관찰과 기록이 함께 이루어집니다.

사용 예:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat 과 Watch

일정한 간격으로 로그를 가져옵니다. 예를 들어 5초마다 요약을 봅니다.

사용 예:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat 과 Timeout

정한 시간이 지나면 logcat을 멈춥니다. 자동 테스트에서 매우 유용합니다.

사용 예:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat 과 Alias

자주 쓰는 명령을 짧게 만듭니다. Termux에서는 .bashrc 또는 .zshrc 파일에 추가합니다.

사용 예 (alias 정의):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

그다음에는 이렇게 부르기만 하면 됩니다:

```
lce
```

---

55- Adb Logcat 과 여러 기기

여러 기기가 연결되어 있을 때 특정 기기를 지정합니다. 먼저 adb devices로 기기 목록을 보고, -s로 하나를 고릅니다.

사용 예:

```
adb devices
```

출력 예:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

사용 예 (특정 기기):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat 과 무선 디버깅

무선 디버깅용입니다. 기기가 같은 Wi-Fi 네트워크에 있으면 케이블 없이 연결됩니다. Android 11부터 지원됩니다.

사용 예 (기기에서 무선 디버깅을 켠 다음):

```
adb pair 192.168.1.42:37123
```

사용 예 (연결):

```
adb connect 192.168.1.42:5555
```

사용 예 (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat 과 Bugreport

시스템 전체의 자세한 보고서를 만듭니다. logcat 외에도 dumpsys, dmesg, top 같은 여러 도구가 실행되고, 모든 것이 하나의 zip에 담깁니다. 가장 포괄적인 디버깅 방법입니다.

사용 예:

```
adb bugreport /Sdcard/Bugreport.Zip
```

사용 예 (컴퓨터로 옮기기):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat 과 Dumpsys

logcat과 함께 시스템 서비스의 현재 상태를 보여 줍니다. 예를 들어 ActivityManager가 그 순간 무엇을 하고 있는지입니다.

사용 예:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat 과 ANR 트레이스

앱이 응답하지 않을 때(ANR) 스택 트레이스를 가져옵니다. ANR 기록은 /Data/Anr/Traces.Txt 파일에 있습니다.

사용 예:

```
adb shell cat /Data/Anr/Traces.Txt
```

사용 예 (root 필요):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat 과 Tombstone

네이티브 비정상 종료용입니다. /Data/Tombstones/ 폴더에 비정상 종료마다 파일이 하나씩 생깁니다. C++나 Rust 코드에 오류가 있을 때 여기를 봅니다.

사용 예:

```
adb shell ls /Data/Tombstones/
```

사용 예 (최신 tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat 과 Systrace

시스템 전체의 성능 추적용입니다. 로그만이 아니라 커널 수준의 추적 데이터를 모읍니다. Perfetto와 함께 씁니다.

사용 예:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat 과 Layout Inspector

화면에 보이는 앱의 레이아웃을 분석합니다. Layout Inspector와 함께 쓰지만, 관련 로그는 logcat으로도 따라갈 수 있습니다.

사용 예:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat 과 Memory Profiler

메모리 사용을 지켜봅니다. Memory Profiler는 보통 Android Studio에 있지만, 메모리 관련 로그는 logcat에서도 보입니다.

사용 예:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat 과 CPU Profiler

CPU 사용을 지켜봅니다. 간단한 측정에는 top 명령으로도 충분합니다.

사용 예:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat 과 Network Profiler

네트워크 트래픽을 지켜봅니다. OkHttp, Retrofit 같은 라이브러리가 logcat에 쓰지만, 완전한 분석에는 Network Profiler가 필요합니다.

사용 예:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat 과 GPU Profiler

GPU 사용을 지켜봅니다. 간단한 측정에는 gfxinfo 명령으로도 충분합니다.

사용 예:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat 과 Battery Historian

배터리 소모를 지켜봅니다. dumpsys battery로도 볼 수 있습니다.

사용 예:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat 으로 Wi-Fi 진단

연결 끊김이나 약한 신호 같은 Wi-Fi 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat 으로 Bluetooth 진단

페어링 실패나 연결 끊김 같은 Bluetooth 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat 으로 센서 진단

가속도계, 자이로스코프, 자력계 같은 센서 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat 으로 카메라 진단

사진 촬영이나 동영상 녹화 같은 카메라 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat 으로 오디오 진단

소리가 나지 않거나 마이크가 작동하지 않는 것 같은 오디오 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat 으로 위치 진단

GPS 신호가 없거나 위치가 갱신되지 않는 것 같은 위치 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat 으로 알림 진단

알림이 나타나지 않거나 엉뚱한 때에 오는 것 같은 알림 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat 으로 권한 진단

권한이 부여되지 않거나 거부되는 것 같은 권한 문제를 디버깅할 때 씁니다.

사용 예:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat 으로 WebView 진단

페이지가 로드되지 않거나 JavaScript 오류가 나는 것 같은 WebView 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s Chromium WebView
```

사용 예 (더 자세히):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat 으로 OkHttp 진단

OkHttp 문제, 즉 네트워크 요청, 응답, 오류를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s Okhttp
```

사용 예 (모든 수준):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat 으로 Retrofit 진단

Retrofit 문제를 디버깅할 때 씁니다. 보통 OkHttp 위에서 동작하므로 OkHttp 로그를 봅니다.

사용 예:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat 으로 Glide 진단

이미지가 로드되지 않거나 메모리 문제가 생기는 것 같은 Glide(이미지 로딩 라이브러리) 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s Glide
```

---

80- Adb Logcat 으로 Picasso 진단

Picasso(이미지 로딩 라이브러리) 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s Picasso
```

---

81- Adb Logcat 으로 Room 진단

Room(데이터베이스 라이브러리) 문제, 즉 쿼리 오류와 스키마 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s Room
```

---

82- Adb Logcat 으로 Firebase 진단

Crashlytics, Analytics, Messaging 같은 모듈에서 Firebase 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat 으로 Crashlytics 진단

Firebase Crashlytics 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat 으로 Analytics 진단

분석 문제, 즉 보내지지 않은 이벤트와 일괄 전송 문제를 디버깅할 때 씁니다.

사용 예:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat 과 ADB 무선 디버깅

무선 디버깅은 가끔 adb를 다시 시작해야 합니다. 먼저 USB로 연결한 다음 무선으로 바꿉니다.

사용 예:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat 과 기기 재시작

기기를 다시 시작하면서 로그를 지켜봅니다. 특히 부팅할 때 생기는 오류를 보기 위해서입니다.

사용 예 (터미널 두 개):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat 과 앱 재시작

앱 하나를 다시 시작하면서 로그를 지켜봅니다.

사용 예:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat 과 앱 설치

APK를 설치하면서 로그를 지켜봅니다.

사용 예:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat 과 앱 제거

앱을 제거하면서 로그를 지켜봅니다.

사용 예:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat 과 스크린숏

스크린숏을 찍고, 오류가 난 순간의 로그를 저장합니다.

사용 예:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat 과 화면 녹화

화면을 녹화하고, 오류가 난 순간의 로그를 저장합니다. Android 4.4부터 지원됩니다.

사용 예:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat 과 Monkey 테스트

무작위 사용자 동작을 흉내 내어 앱을 테스트합니다. 로그와 함께 쓰면 매우 효과적입니다.

사용 예:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat 과 UI Automator 테스트

UI 자동화 테스트용으로, 테스트 중에 로그를 지켜봅니다.

사용 예:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat 과 Espresso 테스트

UI 자동화의 한 부분인 Espresso 테스트용입니다.

사용 예:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat 과 JUnit 테스트

단위 테스트용입니다. 보통 컴퓨터에서 실행하지만 기기에서도 실행할 수 있습니다.

사용 예:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat 과 StrictMode

앱의 성능 문제를 찾아냅니다. 메인 스레드에서 디스크나 네트워크에 접근하면 경고합니다.

사용 예:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat 과 Choreographer

건너뛴 프레임을 찾아냅니다. 애니메이션이 끊길 때 여기를 봅니다.

사용 예:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat 과 GPU 오버드로

오버드로 문제를 찾아냅니다. 그래픽 성능 분석에 씁니다.

사용 예:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat 과 렌더 스레드

렌더 스레드 문제를 찾아냅니다. 최신 Android에서는 그리기가 이 스레드에서 일어납니다.

사용 예:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat 과 GPU 렌더링

GPU 렌더링 문제를 찾아냅니다. GPU 렌더링 프로필을 켠 뒤에 여기를 봅니다.

사용 예:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat 과 TextView

글꼴 문제나 측정 오류 같은 텍스트 뷰 문제를 찾아냅니다.

사용 예:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat 과 RecyclerView

목록 뷰 문제, 즉 스크롤 성능과 뷰 재활용 문제를 찾아냅니다.

사용 예:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat 과 ViewPager

페이지 전환 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat 과 Fragment

Fragment 생명 주기 문제를 찾아냅니다.

사용 예:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat 과 Activity

Activity 생명 주기 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat 과 서비스

백그라운드 서비스 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat 과 WorkManager

백그라운드 작업용 WorkManager 문제를 찾아냅니다.

사용 예:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat 과 AlarmManager

예약 작업용 AlarmManager 문제를 찾아냅니다.

사용 예:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat 과 SharedPreferences

SharedPreferences(간단한 데이터 저장) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat 과 SQLite

SQLite(로컬 데이터베이스) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat 과 파일 작업

파일 읽기와 쓰기 문제를 찾아냅니다.

사용 예:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat 과 네트워크 작업

HTTP, TCP, UDP 같은 네트워크 연결 문제를 찾아냅니다.

사용 예:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat 과 SSL

SSL 인증서 문제를 찾아냅니다. 보안 연결이 실패할 때 여기를 봅니다.

사용 예:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat 과 JSON

JSON 파싱 문제를 찾아냅니다.

사용 예:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat 과 XML

XML 파싱 문제를 찾아냅니다.

사용 예:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat 과 이미지

이미지 로딩과 처리 문제를 찾아냅니다.

사용 예:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat 과 미디어

미디어 재생과 녹화 문제를 찾아냅니다.

사용 예:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat 과 ExoPlayer

ExoPlayer(미디어 플레이어 라이브러리) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat 과 Chromecast

Chromecast 문제를 찾아냅니다.

사용 예:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat 과 광고

광고 라이브러리 문제를 찾아냅니다.

사용 예:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat 과 결제

Google Play 결제 문제를 찾아냅니다.

사용 예:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat 과 지도

지도 라이브러리 문제를 찾아냅니다.

사용 예:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat 과 인증

Google Sign-In, Firebase Auth 같은 로그인 문제를 찾아냅니다.

사용 예:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat 과 알림

알림 보내기와 받기 문제를 찾아냅니다.

사용 예:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat 과 딥 링크

URL로 앱을 여는 것 같은 딥 링크 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat 과 Intent

앱 사이를 오갈 때의 Intent 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat 과 브로드캐스트

시스템과 앱 사이의 브로드캐스트 메시지 문제를 찾아냅니다.

사용 예:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat 과 콘텐츠 제공자

앱 사이에 데이터를 공유하는 콘텐츠 제공자 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat 과 JobScheduler

예약된 백그라운드 작업용 JobScheduler 문제를 찾아냅니다.

사용 예:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat 과 Doze 모드

기기가 잠자는 동안의 백그라운드 작업과 관련된 Doze(절전) 모드 문제를 찾아냅니다.

사용 예:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat 과 배터리 최적화

백그라운드 제한과 관련된 배터리 최적화 문제를 찾아냅니다.

사용 예:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat 과 메모리 누수

메모리 누수를 찾아냅니다. LeakCanary 같은 도구와 함께 씁니다.

사용 예:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat 과 ANR

ANR(앱이 응답하지 않음) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat 과 비정상 종료

비정상 종료를 찾아냅니다.

사용 예:

```
adb logcat -b crash
```

---

135- Adb Logcat 과 네이티브 비정상 종료

네이티브 비정상 종료(C/C++/Rust)를 찾아냅니다.

사용 예:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat 과 Tombstone 분석

네이티브 비정상 종료 기록을 분석합니다.

사용 예:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat 과 Systrace 분석

시스템 추적 기록을 분석합니다.

사용 예:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat 과 Perfetto 분석

최신 시스템 추적 도구로, logcat보다 더 포괄적입니다.

사용 예:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat 과 Atrace 분석

시스템 수준 추적용이며, Perfetto의 옛 형태입니다.

사용 예:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat 과 Traceview 분석

메서드 수준 프로파일링용입니다.

사용 예:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat 과 메서드 추적

특정 메서드를 따라갑니다.

사용 예:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat 과 힙 덤프

메모리 덤프를 만듭니다. 메모리 분석에 매우 유용합니다.

사용 예:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat 과 GC 로그

가비지 컬렉션 이벤트를 지켜봅니다.

사용 예:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat 과 JNI 디버깅

JNI(Java Native Interface) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s JNI
```

---

145- Adb Logcat 과 ART 디버깅

ART(Android Runtime) 문제를 찾아냅니다.

사용 예:

```
adb logcat -s Art
```

---

146- Adb Logcat 과 Dalvik 디버깅

Dalvik 가상 머신 문제를 찾아냅니다. 오래된 Android 버전에서 씁니다.

사용 예:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat 과 Chatty 문제

logcat이 로그를 너무 많이 쓰면 “chatty” 경고를 내고 줄을 건너뜁니다. 필터나 더 큰 버퍼로 해결합니다.

사용 예 (버퍼 늘리기):

```
adb logcat -G 16M
```

사용 예 (필터 쓰기):

```
adb logcat -s MyApp
```

---

148- Adb Logcat 과 컬러 출력

긴 로그 흐름에서 컬러 출력은 눈을 덜 피로하게 하고 오류를 바로 보여 줍니다.

사용 예:

```
adb logcat -v color
```

사용 예 (time과 함께):

```
adb logcat -v time,color
```

---

149- Adb Logcat 과 Bash 스크립트

로그를 자동으로 모으는 스크립트를 쓸 때 씁니다. 그 위에 버그 보고 시스템을 만들 수 있습니다.

스크립트 예:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat 정리

이 목록이 보여 주듯이 logcat은 하나의 명령이 아니라 명령의 모음입니다. 필터, 형식, 버퍼, 파일, 다른 도구와 함께 쓰면 디버깅의 가장 강력한 무기가 됩니다. 상황마다 어떤 조합을 쓸지 알면 버그를 찾는 데 몇 분이면 충분합니다. 기억하세요. 디버깅 세션의 첫 번째 규칙은 언제나 같습니다. 먼저 adb logcat -c로 비우고, 그다음 버그를 재현하고, 그다음 필터를 건 logcat으로 지켜봅니다.

@@ Android Debug Bridge 연결
@: ADB 무선 연결 명령어
1- 무선 디버깅 켜기

먼저 기기에서 개발자 옵션을 켜야 합니다. 이를 위해 설정 → 휴대전화 정보 → 빌드 번호를 7번 누릅니다. 그런 다음 설정 → 시스템 → 개발자 옵션 → 무선 디버깅을 켭니다. 이 단계를 거치지 않으면 adb pair와 adb connect 명령이 모두 작동하지 않습니다.

경로 예:

```
설정 → 휴대전화 정보 → 빌드 번호 (7번 누르기)
설정 → 시스템 → 개발자 옵션 → 무선 디버깅 (켜짐)
```

---

2- 기기의 IP 주소와 포트 찾기

무선 디버깅을 켠 상태에서 “페어링 코드로 기기 페어링” 항목을 엽니다. 그곳에 기기의 IP 주소와 무작위 포트 번호가 나타납니다. 예를 들어 192.168.1.42:37123입니다. 이 포트는 페어링에만 쓰이고 그 뒤에는 바뀝니다. 같은 화면에 6자리 페어링 코드도 있습니다.

화면 예:

```
IP 주소 및 포트
192.168.1.42:37123

Wi-Fi 페어링 코드
482915
```

---

3- Adb Pair 명령

이 명령으로 기기를 페어링합니다. 컴퓨터나 Termux에서 실행합니다. 기기에 표시된 IP 주소와 페어링 포트를 입력합니다. 실행하면 터미널이 Enter pairing code: 라고 묻고 6자리 코드를 기다립니다. 코드가 맞으면 Successfully paired 메시지가 나타납니다.

사용 예:

```
adb pair 192.168.1.42:37123
```

출력 예:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

코드가 틀리면 이렇게 실패합니다:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Adb Pair 에 코드를 넘기는 다른 방법

매번 손으로 코드를 입력할 필요는 없습니다. 어떤 버전은 코드를 매개변수로 바로 받지만, 보안상 보통은 지원하지 않습니다. 그래도 시도해 볼 수는 있습니다.

사용 예 (한 줄로):

```
adb pair 192.168.1.42:37123 482915
```

사용 예 (파이프로):

```
echo "482915" | adb pair 192.168.1.42:37123
```

이 방법들은 버전에 따라 달라집니다. 가장 확실한 방법은 명령을 실행하고 코드를 손으로 입력하는 것입니다.

---

5- Adb Pair 에서 주의할 점

페어링 포트와 연결 포트는 다릅니다. 페어링 포트는 보통 무작위의 큰 번호이고, 연결 포트는 보통 5555입니다. 페어링은 한 번만 하면 되고, 같은 기기에 다시 연결할 때는 adb connect만으로 충분합니다. 하지만 기기를 공장 초기화하거나 무선 디버깅을 끄면 페어링이 지워지고 다시 adb pair가 필요합니다.

비교 예:

```
페어링: adb pair 192.168.1.42:37123   → 코드를 요구, 한 번만
연결:   adb connect 192.168.1.42:5555  → 코드 없음, 매번
```

---

6- Adb Connect 명령

이 명령은 페어링된 기기에 연결합니다. 무선 디버깅 메인 화면에 보이는 IP 주소와 포트를 입력합니다. 포트는 보통 5555이지만 가끔 다릅니다. 연결에 성공하면 Connected to 메시지가 나타납니다.

사용 예:

```
adb connect 192.168.1.42:5555
```

출력 예:

```
Connected to 192.168.1.42:5555
```

이미 연결되어 있으면 이렇게 알려 줍니다:

```
Already connected to 192.168.1.42:5555
```

---

7- 포트 없는 Adb Connect

일부 버전에서는 포트를 지정하지 않으면 기본으로 5555를 시도합니다. 하지만 모든 기기에서 똑같이 동작하지는 않으므로 포트를 명시적으로 쓰는 것이 가장 안전합니다.

사용 예 (기본 포트):

```
adb connect 192.168.1.42
```

사용 예 (명시적 포트):

```
adb connect 192.168.1.42:5555
```

---

8- 연결 확인

연결한 뒤 adb devices 명령으로 기기가 실제로 연결되었는지 확인할 수 있습니다. 기기의 IP 주소가 목록에 나타나고 옆에 device라고 적혀 있어야 합니다. offline이라고 적혀 있으면 연결은 되었지만 아직 안정적이지 않은 것이니 잠시 후 다시 시도하세요.

사용 예:

```
adb devices
```

출력 예:

```
List of devices attached
192.168.1.42:5555   device
```

출력 예 (연결되지 않음):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Adb Disconnect 명령

연결을 끊습니다. 기기가 같은 네트워크에 있어도 adb 연결은 끝납니다. 다시 연결하려면 adb connect를 한 번 더 실행하면 되고, 페어링은 필요 없습니다.

사용 예 (기기 하나):

```
adb disconnect 192.168.1.42:5555
```

출력 예:

```
Disconnected 192.168.1.42:5555
```

사용 예 (모든 기기):

```
adb disconnect
```

출력 예:

```
Disconnected everything
```

---

10- Adb Tcpip 명령

USB로 연결된 기기를 무선 모드로 바꿉니다. 이 명령은 기기가 지정한 포트(보통 5555)에서 대기하게 합니다. adb pair가 없는 Android 11 이전 버전에서 더 흔히 쓰는 방법입니다.

사용 예:

```
adb tcpip 5555
```

출력 예:

```
Restarting in TCP mode port: 5555
```

그다음 USB 케이블을 빼고 이 명령을 실행합니다:

```
adb connect 192.168.1.42:5555
```

---

11- Adb Usb 명령

무선 모드에서 USB 모드로 돌아갑니다. 기기를 다시 케이블로 연결하고 싶을 때 씁니다.

사용 예:

```
adb usb
```

출력 예:

```
Restarting in USB mode
```

---

12- Adb Kill-Server 와 Start-Server

가끔 adb 서버가 멈추거나 연결을 거부합니다. 그럴 때는 서버를 멈추고 다시 시작해야 합니다. 이렇게 하면 모든 연결이 초기화되고 깨끗하게 새로 시작합니다.

사용 예:

```
adb kill-server
adb start-server
```

출력 예:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

그다음 다시 연결합니다:

```
adb connect 192.168.1.42:5555
```

---

13- 같은 네트워크에 있기

무선 adb를 쓰려면 기기와 컴퓨터 또는 휴대전화가 같은 Wi-Fi 네트워크에 있어야 합니다. 서로 다른 네트워크에서는 연결되지 않습니다. 게스트 네트워크, 5 GHz와 2.4 GHz의 분리, VPN이나 방화벽이 가끔 문제를 일으킵니다. 간단한 해결책은 두 기기를 같은 공유기에 연결하고 VPN을 끄는 것입니다.

확인 예:

```
기기: 설정 → Wi-Fi → 연결된 네트워크 (예: Home_Wifi)
Termux: 설정 → Wi-Fi → 연결된 네트워크 (예: Home_Wifi)
```

---

14- IP 주소를 직접 찾기

기기의 IP 주소를 찾으려면 설정 → Wi-Fi → 연결된 네트워크 → 네트워크 세부정보로 갑니다. 또는 아래 명령으로 Termux에서 봅니다. 하지만 IP 주소는 시간이 지나면 바뀔 수 있으므로, 매번 무선 디버깅 화면을 확인하는 것이 가장 확실합니다.

사용 예 (Termux):

```
ifconfig wlan0
```

사용 예 (기기에서):

```
설정 → Wi-Fi → 연결된 네트워크 → IP 주소
```

---

15- 같은 기기의 Termux 에서 자기 자신에 연결하기

Termux가 같은 기기에서 돌아가면 adb는 이미 로컬이지만, 그래도 adb connect로 localhost를 통해 연결할 수 있습니다. 보통 127.0.0.1:5555를 씁니다. 일부 기기는 이를 거부하는데, 그럴 때는 기기 자신의 로컬 IP를 씁니다.

사용 예 (localhost):

```
adb connect 127.0.0.1:5555
```

사용 예 (자신의 로컬 IP):

```
adb connect 192.168.1.42:5555
```

---

16- 여러 기기가 연결되었을 때 명령 실행

여러 기기가 동시에 연결되어 있다면 명령마다 -s 매개변수로 기기를 고르세요. 그렇지 않으면 adb가 error: more than one device로 실패합니다.

사용 예:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- 연결이 끊겼을 때

Wi-Fi 신호가 약해지거나, 기기가 절전 상태가 되거나, 네트워크가 바뀌면 연결이 끊깁니다. 그러면 기기가 adb devices 목록에서 사라지거나 offline으로 보입니다. 해결책은 adb connect를 다시 실행하는 것입니다. 페어링은 보통 그대로 유효하므로 adb pair는 필요 없습니다.

복구 예:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- 기기를 다시 시작한 뒤

기기를 다시 시작하면 무선 디버깅은 보통 꺼집니다. 일부 기기에서는 켜진 채로 남지만 페어링은 지워집니다. 기기가 부팅되면 무선 디버깅 화면으로 가서 새 포트와 새 코드를 확인하고, adb pair를 다시 실행한 다음 adb connect를 실행합니다.

흐름 예:

```
기기 재시작됨 → 무선 디버깅 켜기
새 포트: 42157, 새 코드: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- 페어링 중 자주 나는 오류

Failed: Wrong password → 코드를 잘못 입력했습니다.
Failed to connect: Connection refused → 포트가 틀렸거나 기기가 대기하고 있지 않습니다.
Failed: Pairing rejected → 기기가 페어링을 거부했습니다. 코드를 새로 받으세요.
Timeout → 네트워크에 문제가 있습니다. 같은 네트워크에 있는지 확인하세요.

오류와 해결 예:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

해결: 기기의 코드가 바뀌었습니다. 새 코드를 확인하고 다시 시도하세요.

---

20- 무선 Adb 끄기

작업이 끝나면 무선 디버깅을 끄는 것이 보안에 좋습니다. 설정에서 끄거나 adb로 USB 모드로 바꿀 수 있습니다.

사용 예:

```
adb usb
adb disconnect
```

사용 예 (설정에서):

```
설정 → 시스템 → 개발자 옵션 → 무선 디버깅 (꺼짐)
```

---

21- Android 버전에 따른 차이

Android 11 이상: adb pair로 페어링하는 것이 필수이며 무작위 포트를 씁니다.
Android 10 이하: adb tcpip 5555로 무선 모드로 바꿉니다. 페어링은 없습니다.

비교 예:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- 보안 참고 사항

무선 디버깅이 켜져 있는 동안 같은 네트워크의 악의적인 사람이 기기에 접근하려 할 수 있습니다. 그러니 믿을 수 없는 Wi-Fi 네트워크에서는 이 기능을 꺼 두세요. 페어링 코드는 보안을 위해 있지만 그래도 조심하는 것이 좋습니다. 작업이 끝나면 끄세요.

예방 예:

```
쓰지 않을 때: 설정 → 개발자 옵션 → 무선 디버깅 (꺼짐)
안전한 네트워크에서: 집이나 사무실에서만 켜기
```

---

23- 전체 흐름을 한곳에

아래 스크립트는 처음부터 연결까지의 모든 단계를 보여 줍니다. IP와 포트를 자신의 기기 값으로 바꾸기만 하면 됩니다.

흐름 예:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

예상 결과:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- 지속적인 연결을 위한 단축 스크립트

자주 연결한다면 스크립트를 써서 매번 명령 하나로 연결할 수 있습니다. 예를 들어 Termux에서는 ~/.bashrc 파일에 함수를 추가할 수 있습니다.

함수 예:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "기기에 표시된 IP:포트와 코드를 입력하세요:"
    read -p "주소: " ADRES
    adb pair "$ADRES"
    read -p "연결 포트 (예: 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

사용법:

```
wifi_adb
```

---

25- 마치며

무선 adb를 쓰면 USB 케이블 없이 디버깅할 수 있고, 휴대전화 자체의 Termux에서 작업할 때 특히 편리합니다. 기본 명령 두 개는 adb pair와 adb connect입니다. 앞의 것은 한 번만 페어링하고, 뒤의 것은 연결할 때마다 씁니다. 나머지(adb devices, adb disconnect, adb tcpip, adb kill-server)는 이 두 명령을 돕고 문제를 해결하는 데 씁니다. 같은 네트워크 규칙을 잊지 말고, 쓰지 않을 때는 끄고, 무언가 잘못되면 서버를 다시 시작하세요. 이것으로 끝입니다.
"#;

const HINDI: &str = r#"@@ Logcat कमांड
1- Adb Logcat

डिवाइस के सभी लॉग लाइव टर्मिनल पर भेजता है। कमांड चलने के क्षण से सिस्टम, ऐप, सेवाएँ और कर्नेल जो भी लॉग लाइन बनाते हैं, वह स्क्रीन पर दिखती है। रोकने के लिए कीबोर्ड पर Ctrl + C दबाएँ। यह कमांड डिबगिंग की नींव है; लेकिन फ़िल्टर के बिना यह हर सेकंड सैकड़ों लाइनें बहा सकता है और ज़रूरी चीज़ ढूँढना मुश्किल हो जाता है।

उपयोग का उदाहरण:

```
adb logcat
```

आउटपुट का उदाहरण:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

अब तक जमा हुए लॉग दिखाता है और कमांड ख़त्म हो जाता है। लगातार बहाव के बजाय उस पल की एक तस्वीर लेता है। बग रिपोर्ट तैयार करते समय, क्रैश के बाद स्थिति सहेजते समय, या किसी स्क्रिप्ट से एक बार लॉग लेने के लिए इसका उपयोग करें।

उपयोग का उदाहरण:

```
adb logcat -d
```

उपयोग का उदाहरण (फ़ाइल में सहेजना):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

यह कमांड उन स्क्रिप्ट में बहुत इस्तेमाल होता है जो अपने आप सर्वर पर रिपोर्ट भेजती हैं, क्योंकि यह ख़ुद ख़त्म हो जाता है और स्क्रिप्ट को कभी अटकाता नहीं।

---

3- Adb Logcat -C

सभी लॉग बफ़र साफ़ करता है और उस पल तक जमा सब कुछ मिटा देता है। अगर आप इसे नए डिबग सत्र से पहले चलाते हैं, तो आपको केवल उसके बाद बने लॉग दिखेंगे और पुराना शोर आपको भ्रमित नहीं करेगा। किसी बग को दोहराने की कोशिश में यह आमतौर पर पहला कदम होता है।

उपयोग का उदाहरण:

```
adb logcat -c
adb logcat
```

ऊपर के दो कमांड क्रम से चलाने पर पूरी तरह साफ़ लॉग प्रवाह मिलता है। केवल सफ़ाई के बाद होने वाली चीज़ें दिखती हैं।

---

4- Adb Logcat -G

मौजूदा लॉग बफ़र का आकार और वे कितने भरे हैं, यह दिखाता है। हर बफ़र की भराई, कुल क्षमता और इस्तेमाल हुई मात्रा को लाइन-दर-लाइन सूचीबद्ध करता है। अगर लॉग बहुत तेज़ी से बह रहे हैं और पुरानी प्रविष्टियाँ हटाई जा रही हैं (इसे «chatty» कहते हैं), तो पहले यहाँ देखें।

उपयोग का उदाहरण:

```
adb logcat -g
```

आउटपुट का उदाहरण:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G आकार

लॉग बफ़र का आकार बदलता है। उदाहरण के लिए, 256 KiB के बफ़र को 16 MiB तक बढ़ाया जा सकता है। इससे ज़्यादा लॉग रखे जाते हैं और पुरानी प्रविष्टियाँ देर से हटती हैं। लंबे परीक्षणों में या बहुत शोर करने वाले ऐप को डिबग करते समय बहुत उपयोगी है। डिवाइस रीस्टार्ट होने पर सेटिंग वापस पहले जैसी हो जाती है।

उपयोग का उदाहरण:

```
adb logcat -G 16M
```

उपयोग का उदाहरण (छोटा करना):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F फ़ाइल

लॉग को टर्मिनल पर दिखाने के बजाय बताई गई फ़ाइल में लिखता है। टर्मिनल साफ़ रहता है और लॉग सुरक्षित रहता है। फ़ाइल का पाथ डिवाइस पर होता है, कंप्यूटर पर नहीं। मोबाइल टर्मिनल (Termux) में आमतौर पर /Sdcard/ में लिखा जाता है।

उपयोग का उदाहरण:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

उपयोग का उदाहरण (फ़ाइल में जोड़ते हुए):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

ऊपर के उदाहरण में फ़ाइल 4 MiB तक पहुँचने पर नई फ़ाइल खुलती है, और कुल 10 फ़ाइलें रखी जाती हैं।

---

7- Adb Logcat -B Main

केवल ऐप के मुख्य बफ़र को देखता है। यहाँ वे लॉग होते हैं जो ऐप Log.d, Log.i और Log.e जैसे कॉल से लिखते हैं। यह पहले से डिफ़ॉल्ट रूप से देखा जाता है, लेकिन जब केवल यही बफ़र चाहिए तब इसे स्पष्ट रूप से लिखा जाता है। ऐप बनाते समय सबसे ज़्यादा यहीं देखा जाता है।

उपयोग का उदाहरण:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

सिस्टम सेवाओं के लॉग देखता है। यहाँ ActivityManager, WindowManager, PackageManager और PowerManager जैसी सेवाओं के लॉग होते हैं। ऐप शुरू होना, अनुमति देना और सेवा से जुड़ना जैसी घटनाएँ देखने के काम आता है। आमतौर पर main के साथ देखा जाता है।

उपयोग का उदाहरण:

```
adb logcat -b system
```

आउटपुट का उदाहरण:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

केवल क्रैश लॉग देखता है। यहाँ वे FATAL EXCEPTION ब्लॉक होते हैं जो ऐप के ज़बरदस्ती बंद होने पर लिखे जाते हैं। क्रैश डिबग करते समय पहले यही बफ़र देखना सबसे तेज़ रास्ता है, क्योंकि स्टैक ट्रेस सीधे यहीं होता है।

उपयोग का उदाहरण:

```
adb logcat -b crash
```

आउटपुट का उदाहरण:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

रेडियो, टेलीफ़ोनी, सिग्नल और डेटा कनेक्शन के लॉग देखता है। SIM कार्ड, नेटवर्क पंजीकरण, 4G/5G कनेक्शन, SMS या कॉल से जुड़ा काम करते समय उपयोगी है। सामान्य ऐप विकास में आमतौर पर ज़रूरत नहीं होती।

उपयोग का उदाहरण:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

सिस्टम घटनाओं के बाइनरी लॉग देखता है। इस बफ़र में सामान्य टेक्स्ट नहीं, बल्कि Log.event कॉल से छोड़े गए छोटे एन्कोडेड रिकॉर्ड होते हैं। ये am_proc_start और am_activity_launch जैसे टैग के साथ दिखते हैं। प्रदर्शन विश्लेषण में इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

कर्नेल लॉग देखता है। Linux कर्नेल के लिखे संदेश दिखाता है, जो आमतौर पर dmesg कमांड के आउटपुट जैसे दिखते हैं। यहाँ ड्राइवर की त्रुटियाँ, मेमोरी की समस्याएँ और तापमान व पावर प्रबंधन की जानकारी होती है। root की ज़रूरत पड़ सकती है।

उपयोग का उदाहरण:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

सभी बफ़र एक साथ देखता है। main, system, crash, radio, events और kernel मिले-जुले बहते हैं। सब कुछ देखना हो तो उपयोगी है, लेकिन आउटपुट बहुत भारी हो जाता है, इसलिए आमतौर पर फ़िल्टर के साथ जोड़ा जाता है।

उपयोग का उदाहरण:

```
adb logcat -b all
```

उपयोग का उदाहरण (फ़िल्टर के साथ):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B कई बफ़र

कॉमा से अलग किए गए कई बफ़र एक साथ देखता है, जैसे ऐप और क्रैश के लॉग साथ में देखना। केवल ज़रूरी चैनल चुनकर आप आउटपुट कम करते हैं।

उपयोग का उदाहरण:

```
adb logcat -b main,crash
```

उपयोग का उदाहरण (तीन चैनल):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

डिफ़ॉल्ट छोटा आउटपुट फ़ॉर्मेट। हर लाइन केवल प्राथमिकता और टैग से शुरू होती है। जल्दी नज़र डालने के लिए अच्छा है, लेकिन टाइमस्टैम्प न होने से घटनाओं का क्रम समझना मुश्किल है।

उपयोग का उदाहरण:

```
adb logcat -v brief
```

आउटपुट का उदाहरण:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

तारीख़ और समय जोड़ता है। छोटे फ़ॉर्मेट से कहीं ज़्यादा उपयोगी है, क्योंकि आप देख सकते हैं कि हर घटना कब हुई। सेकंड के बाद मिलीसेकंड भी दिखते हैं।

उपयोग का उदाहरण:

```
adb logcat -v time
```

आउटपुट का उदाहरण:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

सबसे ज़्यादा सुझाया जाने वाला आउटपुट फ़ॉर्मेट। इसमें तारीख़, समय, प्राथमिकता, टैग, PID (प्रोसेस पहचानकर्ता) और TID (थ्रेड पहचानकर्ता) होते हैं। कई थ्रेड वाले ऐप में यह जानने के लिए ज़रूरी है कि किस थ्रेड ने क्या किया। Android Studio भी डिफ़ॉल्ट रूप से यही फ़ॉर्मेट इस्तेमाल करता है।

उपयोग का उदाहरण:

```
adb logcat -v threadtime
```

आउटपुट का उदाहरण:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

सभी मेटाडेटा फ़ील्ड के साथ विस्तृत आउटपुट देता है और प्रविष्टियों के बीच एक ख़ाली लाइन रखता है। लंबे सत्रों में हर लॉग को एक ब्लॉक की तरह दिखाता है, लेकिन जगह घेरता है।

उपयोग का उदाहरण:

```
adb logcat -v long
```

आउटपुट का उदाहरण:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

प्राथमिकता के अनुसार लॉग को रंग देता है। त्रुटियाँ लाल, चेतावनियाँ नारंगी, जानकारी हरी और डिबग नीले रंग में। लंबे प्रवाह में नज़र सीधे त्रुटि पर जाती है। टर्मिनल को रंग सपोर्ट करने चाहिए।

उपयोग का उदाहरण:

```
adb logcat -v color
```

उपयोग का उदाहरण (time के साथ):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

समय फ़ॉर्मेट में साल जोड़ता है। एक साल से ज़्यादा फैले लंबे लॉग या संग्रह में इस्तेमाल होता है। सामान्य विकास में आमतौर पर ज़रूरत नहीं होती।

उपयोग का उदाहरण:

```
adb logcat -v year
```

आउटपुट का उदाहरण:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

लॉग बनाने वाले ऐप का उपयोगकर्ता पहचानकर्ता (UID) दिखाता है। जब कई ऐप एक ही टैग इस्तेमाल करते हैं तब उपयोगी है। यह एक संशोधक है, इसलिए दूसरे फ़ॉर्मेट के साथ जोड़ा जाता है।

उपयोग का उदाहरण:

```
adb logcat -v uid
```

आउटपुट का उदाहरण:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat टैग और प्राथमिकता फ़िल्टर

किसी ख़ास टैग के लॉग किसी ख़ास प्राथमिकता से ऊपर दिखाता है। प्राथमिकताएँ हैं: V (Verbose) सबसे नीची, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) और S (Silent) सबसे ऊँची। फ़िल्टर करने का यह तरीका logcat की सबसे ताक़तवर सुविधा है।

उपयोग का उदाहरण (केवल त्रुटियाँ):

```
adb logcat *:E
```

उपयोग का उदाहरण (चेतावनी और ऊपर):

```
adb logcat *:W
```

उपयोग का उदाहरण (अनुमति सूची):

```
adb logcat ActivityManager:I MyApp:D *:S
```

आख़िरी उदाहरण ActivityManager को info से और MyApp को debug से ऊपर दिखाता है, और बाकी सभी टैग को चुप कर देता है। कुछ शेल में * अक्षर विशेष होता है, इसलिए इसे उद्धरण चिह्नों में रखना पड़ता है:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S टैग

मौन फ़िल्टर का शॉर्टकट। केवल बताया गया टैग दिखाता है और बाकी सब कुछ चुप कर देता है। विकास के दौरान अपने टैग पर ध्यान देने का सबसे आसान तरीका।

उपयोग का उदाहरण:

```
adb logcat -s MyApp
```

उपयोग का उदाहरण (कई टैग):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

ऊपर के उदाहरण में केवल तीन टैग के लॉग गुज़रते हैं; बाकी सब चुप रहता है।

---

24- Adb Logcat -S प्राथमिकता के साथ

टैग और प्राथमिकता को एक साथ बताने के काम आता है, जैसे MyApp टैग की केवल चेतावनी और ऊपर की प्रविष्टियाँ देखना।

उपयोग का उदाहरण:

```
adb logcat MyApp:W *:S
```

यह कमांड MyApp टैग वाले केवल warning, error और fatal लॉग दिखाता है।

---

25- Adb Logcat --Pid फ़िल्टर

किसी ख़ास प्रोसेस पहचानकर्ता (PID) के ही लॉग दिखाता है। किसी ऐप का PID जानने के लिए पहले adb shell pidof चलाया जाता है और फिर उस PID से फ़िल्टर किया जाता है। एक सत्र के सभी लॉग अलग करने के लिए बिल्कुल सही।

उपयोग का उदाहरण (PID पाना):

```
adb shell pidof com.example.app
```

आउटपुट का उदाहरण:

```
5678
```

उपयोग का उदाहरण (फ़िल्टर करना):

```
adb logcat --pid=5678
```

उपयोग का उदाहरण (एक लाइन में):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T समय

बताए गए टाइमस्टैम्प से आगे के लॉग दिखाता है, यानी «इस पल के बाद का लाओ»। बड़े T से लिखा जाता है; छोटे t का मतलब कुछ और है।

उपयोग का उदाहरण:

```
adb logcat -T "09-21 14:00:00.000"
```

उपयोग का उदाहरण (आज से):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T लाइनों की संख्या

छोटे t के साथ बताई गई संख्या में सबसे नई लाइनें दिखाता है और कमांड ख़त्म हो जाता है। -d की तरह चलता है, लेकिन केवल आख़िरी N लाइनें देता है। जल्दी नज़र डालने के लिए बढ़िया।

उपयोग का उदाहरण (आख़िरी 100 लाइनें):

```
adb logcat -t 100
```

उपयोग का उदाहरण (आख़िरी 500 लाइनें और एक फ़िल्टर):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

रेगुलर एक्सप्रेशन (regex) से सामग्री में खोजता है और मेल खाने वाली लाइनें दिखाता है। तेज़ टेक्स्ट खोज के लिए बहुत ताक़तवर है; ख़ासकर एक साथ कई शब्द खोजते समय फ़िल्टर से तेज़ है।

उपयोग का उदाहरण (एक शब्द):

```
adb logcat -e "Login"
```

उपयोग का उदाहरण (कई शब्द):

```
adb logcat -e "Login|Error|Crash"
```

उपयोग का उदाहरण (पैकेज नाम के साथ):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B बाइनरी आउटपुट

लॉग को बाइनरी फ़ॉर्मेट में लिखता है। इंसान इसे पढ़ नहीं सकते, लेकिन दूसरा logcat इसे फिर से पढ़ सकता है या दूसरे टूल इसे प्रोसेस कर सकते हैं। स्वचालित विश्लेषण प्रणालियों में इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat -B > Ham_Log.Bin
```

बाद में वह फ़ाइल ऐसे पढ़ी जाती है:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S आँकड़े

लॉग के आँकड़े दिखाता है: हर टैग ने कितने लॉग बनाए और किन टैग ने «chatty» चेतावनी दी। अगर आपको लगता है कि कोई ऐप लॉग में बाढ़ ला रहा है, तो यह कमांड इसकी पुष्टि करता है।

उपयोग का उदाहरण:

```
adb logcat -S
```

आउटपुट का उदाहरण:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R और -N एक साथ

घूमने वाले (rotating) लॉग के लिए। -r हर फ़ाइल का आकार तय करता है और -n बताता है कि कितनी फ़ाइलें रखी जाएँ। जब कोई ऐप लंबे समय तक चलेगा और आप लगातार लॉग रखना चाहते हैं तब उपयोगी है। पुरानी फ़ाइलें मिटती हैं और नई खुलती हैं।

उपयोग का उदाहरण:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

ऊपर के उदाहरण में हर बार कोई फ़ाइल 4 MiB तक पहुँचने पर नई फ़ाइल खुलती है और कुल 10 फ़ाइलें रखी जाती हैं। इस तरह डिस्क कभी बेहिसाब नहीं भरती।

---

32- Adb Logcat -P फ़िल्टर स्क्रिप्ट

logcat के लिए एक कस्टम फ़िल्टर स्क्रिप्ट बताता है। यह Android का अपना फ़िल्टर फ़ॉर्मेट है और आमतौर पर सिस्टम डेवलपर इसका उपयोग करते हैं। ऐप डेवलपर इसे कम ही इस्तेमाल करते हैं।

उपयोग का उदाहरण:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L टाइमस्टैम्प

एक विकल्प जो कुछ फ़ॉर्मेट में हर लाइन की शुरुआत में टाइमस्टैम्प जोड़ता है। व्यवहार Android संस्करणों के बीच बदलता है। आमतौर पर -v time जैसा ही काम करता है, लेकिन अलग तरीके से।

उपयोग का उदाहरण:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

हर लाइन को printf फ़ॉर्मेट में दिखाता है, यानी प्राथमिकता (PID) टैग: संदेश। पुराने टूल के साथ संगतता के लिए है, लेकिन आधुनिक Android में शायद ही ज़रूरत पड़ती है।

उपयोग का उदाहरण:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

बिना मेटाडेटा के केवल कच्चा संदेश दिखाता है। केवल लॉग लिखा जाता है, बिना टैग, प्राथमिकता या टाइमस्टैम्प के। आउटपुट को किसी दूसरे प्रोग्राम को देने के लिए उपयोगी है।

उपयोग का उदाहरण:

```
adb logcat -v raw
```

आउटपुट का उदाहरण:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

हर लाइन की शुरुआत में केवल टैग लिखता है और फिर बाकी। प्राथमिकता और PID छिपे रहते हैं। जब आप एक टैग पर ध्यान दे रहे हों तब सुविधाजनक है।

उपयोग का उदाहरण:

```
adb logcat -v tag
```

आउटपुट का उदाहरण:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

हर लाइन की शुरुआत में पहले PID और फिर टैग लिखता है। प्रोसेस के हिसाब से विश्लेषण के लिए सुविधाजनक है; जल्दी दिखता है कि किस प्रोसेस ने क्या लिखा।

उपयोग का उदाहरण:

```
adb logcat -v process
```

आउटपुट का उदाहरण:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

हर लाइन की शुरुआत में केवल थ्रेड पहचानकर्ता (TID) लिखता है और फिर बाकी। कई थ्रेड वाले विश्लेषण में दिखाता है कि कौन सा थ्रेड चल रहा है। एक छोटा फ़ॉर्मेट।

उपयोग का उदाहरण:

```
adb logcat -v thread
```

आउटपुट का उदाहरण:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D और -T एक साथ

किसी समय-सीमा के लॉग लेता है, जैसे पिछले एक घंटे के, ताकि उन्हें फ़ाइल में सहेजा जा सके।

उपयोग का उदाहरण:

```
adb logcat -d -T "09-21 13:00:00.000"
```

उपयोग का उदाहरण (फ़ाइल में):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C और -D क्रम से

एक प्रवाह जो पहले साफ़ करता है और फिर इकट्ठा करता है। जैसे परीक्षण शुरू होने से पहले साफ़ करना और परीक्षण के अंत में लॉग इकट्ठा करना।

उपयोग का उदाहरण:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat को फ़ाइल में सहेजकर कंप्यूटर पर ले जाना

मोबाइल टर्मिनल में लंबे समय तक लॉग रखने के लिए सुविधाजनक। पहले डिवाइस पर फ़ाइल में लिखा जाता है और फिर adb pull से कंप्यूटर पर ले जाया जाता है।

उपयोग का उदाहरण (डिवाइस पर सहेजना):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

उपयोग का उदाहरण (कंप्यूटर पर ले जाना):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat के साथ Grep

मोबाइल टर्मिनल (Termux) में grep से तेज़ी से टेक्स्ट खोजने के लिए इस्तेमाल होता है। यह regex फ़िल्टर से ज़्यादा लचीला हो सकता है, क्योंकि grep के सभी विकल्प मिलते हैं।

उपयोग का उदाहरण:

```
adb logcat | grep "Hata"
```

उपयोग का उदाहरण (बड़े-छोटे अक्षर का भेद किए बिना):

```
adb logcat | grep -I "error"
```

उपयोग का उदाहरण (लाइन नंबर के साथ):

```
adb logcat | grep -N "Exception"
```

उपयोग का उदाहरण (कई शब्द):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat के साथ Tail

लाइव प्रवाह को उसकी आख़िरी लाइनों तक सीमित करता है, जब आप पूरा लॉग नहीं, केवल अंत देखना चाहते हैं।

उपयोग का उदाहरण:

```
adb logcat | tail -100
```

उपयोग का उदाहरण (लगातार पीछा करना):

```
adb logcat | tail -F
```

---

44- Adb Logcat के साथ Head

पहली लाइनें दिखाता है। किसी लंबे लॉग पर जल्दी नज़र डालने के लिए अच्छा।

उपयोग का उदाहरण:

```
adb logcat -d | head -50
```

---

45- Adb Logcat के साथ Wc

लाइनों की कुल संख्या दिखाता है, जैसे यह मापने के लिए कि किसी परीक्षण ने कितने लॉग बनाए।

उपयोग का उदाहरण:

```
adb logcat -d | wc -l
```

उपयोग का उदाहरण (केवल त्रुटियाँ गिनना):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat के साथ Sort

लॉग को वर्णक्रम या संख्या के क्रम में लगाता है। ज़्यादातर विश्लेषण के दौरान इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat -d | sort
```

उपयोग का उदाहरण (उल्टे क्रम में):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat के साथ Uniq

दोहराई गई लाइनें हटाता है। अगर कोई ऐप बार-बार एक ही त्रुटि लिखता है, तो यह कमांड आउटपुट को सरल बनाता है।

उपयोग का उदाहरण:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

ऊपर का उदाहरण सबसे ज़्यादा दोहराई गई 20 लाइनें गिनकर दिखाता है। डिबगिंग में बहुत उपयोगी।

---

48- Adb Logcat के साथ Awk

ख़ास कॉलम निकालता है या टेक्स्ट को प्रोसेस करता है, जैसे threadtime फ़ॉर्मेट में PID कॉलम।

उपयोग का उदाहरण (केवल PID कॉलम):

```
adb logcat -v threadtime | awk '{print $3}'
```

उपयोग का उदाहरण (केवल टैग और संदेश):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat के साथ Xargs

हर लाइन के लिए एक कमांड चलाता है, जैसे हर त्रुटि लाइन पर एक सूचना भेजना। ज़्यादातर स्वचालन में इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat के साथ Sed

टेक्स्ट बदलता है। आउटपुट में ख़ास पैटर्न बदलने के लिए बढ़िया।

उपयोग का उदाहरण:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

ऊपर का उदाहरण हर MyApp की जगह Uygulama लिखता है।

---

51- Adb Logcat के साथ Tee

एक ही समय में स्क्रीन पर दिखाता है और फ़ाइल में लिखता है। लाइव देखना और रिकॉर्ड करना साथ-साथ होते हैं।

उपयोग का उदाहरण:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat के साथ Watch

तय अंतराल पर लॉग लेता है, जैसे हर 5 सेकंड में एक सारांश।

उपयोग का उदाहरण:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat के साथ Timeout

तय समय के बाद logcat को रोक देता है। स्वचालित परीक्षणों में बहुत उपयोगी।

उपयोग का उदाहरण:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat के साथ Alias

अक्सर इस्तेमाल होने वाले कमांड को छोटा करता है। Termux में इसे .bashrc या .zshrc फ़ाइल में जोड़ा जाता है।

उपयोग का उदाहरण (alias बनाना):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

फिर बस इसे ऐसे बुलाएँ:

```
lce
```

---

55- Adb Logcat के साथ कई डिवाइस

कई डिवाइस जुड़े होने पर किसी ख़ास डिवाइस को चुनता है। पहले adb devices से डिवाइस की सूची देखें और फिर -s से एक चुनें।

उपयोग का उदाहरण:

```
adb devices
```

आउटपुट का उदाहरण:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

उपयोग का उदाहरण (एक ख़ास डिवाइस):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat के साथ वायरलेस डिबगिंग

वायरलेस डिबगिंग के लिए। जब डिवाइस उसी Wi-Fi नेटवर्क पर हो, तो बिना केबल के जुड़ा जाता है। Android 11 से समर्थित।

उपयोग का उदाहरण (डिवाइस पर वायरलेस डिबगिंग चालू करें, फिर):

```
adb pair 192.168.1.42:37123
```

उपयोग का उदाहरण (कनेक्शन):

```
adb connect 192.168.1.42:5555
```

उपयोग का उदाहरण (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat के साथ Bugreport

पूरे सिस्टम की विस्तृत रिपोर्ट बनाता है। logcat के अलावा dumpsys, dmesg और top जैसे कई टूल चलते हैं और सब कुछ एक zip में आ जाता है। डिबगिंग का सबसे व्यापक तरीका।

उपयोग का उदाहरण:

```
adb bugreport /Sdcard/Bugreport.Zip
```

उपयोग का उदाहरण (कंप्यूटर पर ले जाना):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat के साथ Dumpsys

logcat के साथ सिस्टम सेवाओं की मौजूदा स्थिति दिखाता है, जैसे ActivityManager उस पल क्या कर रहा है।

उपयोग का उदाहरण:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat के साथ ANR ट्रेस

जब कोई ऐप जवाब नहीं देता (ANR) तब स्टैक ट्रेस लेता है। ANR रिकॉर्ड /Data/Anr/Traces.Txt फ़ाइल में रहते हैं।

उपयोग का उदाहरण:

```
adb shell cat /Data/Anr/Traces.Txt
```

उपयोग का उदाहरण (root ज़रूरी):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat के साथ Tombstone

नेटिव क्रैश के लिए। /Data/Tombstones/ फ़ोल्डर में हर क्रैश के लिए एक फ़ाइल बनती है। जब C++ या Rust कोड में त्रुटि हो तब यहाँ देखा जाता है।

उपयोग का उदाहरण:

```
adb shell ls /Data/Tombstones/
```

उपयोग का उदाहरण (सबसे नया tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat के साथ Systrace

पूरे सिस्टम के प्रदर्शन ट्रेसिंग के लिए। केवल लॉग नहीं, कर्नेल स्तर का ट्रेस डेटा इकट्ठा करता है। Perfetto के साथ इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat के साथ Layout Inspector

स्क्रीन पर किसी ऐप के लेआउट का विश्लेषण करता है। Layout Inspector के साथ इस्तेमाल होता है, लेकिन संबंधित लॉग logcat से भी देखे जा सकते हैं।

उपयोग का उदाहरण:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat के साथ Memory Profiler

मेमोरी का उपयोग देखता है। Memory Profiler आमतौर पर Android Studio में होता है, लेकिन मेमोरी से जुड़े लॉग logcat में भी दिखते हैं।

उपयोग का उदाहरण:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat के साथ CPU Profiler

CPU का उपयोग देखता है। सरल माप के लिए top कमांड भी काफ़ी है।

उपयोग का उदाहरण:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat के साथ Network Profiler

नेटवर्क ट्रैफ़िक देखता है। OkHttp और Retrofit जैसी लाइब्रेरी logcat में लिखती हैं, लेकिन पूरे विश्लेषण के लिए Network Profiler चाहिए।

उपयोग का उदाहरण:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat के साथ GPU Profiler

GPU का उपयोग देखता है। सरल माप के लिए gfxinfo कमांड भी काफ़ी है।

उपयोग का उदाहरण:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat के साथ Battery Historian

बैटरी की खपत देखता है। इसे dumpsys battery से भी देखा जा सकता है।

उपयोग का उदाहरण:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat से Wi-Fi की जाँच

Wi-Fi की समस्याएँ डिबग करने के लिए, जैसे कनेक्शन टूटना या कमज़ोर सिग्नल।

उपयोग का उदाहरण:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat से Bluetooth की जाँच

Bluetooth की समस्याएँ डिबग करने के लिए, जैसे पेयरिंग विफल होना या कनेक्शन टूटना।

उपयोग का उदाहरण:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat से सेंसर की जाँच

एक्सेलेरोमीटर, जाइरोस्कोप और मैग्नेटोमीटर जैसे सेंसर की समस्याएँ डिबग करने के लिए।

उपयोग का उदाहरण:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat से कैमरे की जाँच

कैमरे की समस्याएँ डिबग करने के लिए, जैसे फ़ोटो लेना या वीडियो रिकॉर्ड करना।

उपयोग का उदाहरण:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat से ऑडियो की जाँच

ऑडियो की समस्याएँ डिबग करने के लिए, जैसे आवाज़ न आना या माइक्रोफ़ोन काम न करना।

उपयोग का उदाहरण:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat से लोकेशन की जाँच

लोकेशन की समस्याएँ डिबग करने के लिए, जैसे GPS सिग्नल न मिलना या लोकेशन अपडेट न होना।

उपयोग का उदाहरण:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat से सूचनाओं की जाँच

सूचनाओं की समस्याएँ डिबग करने के लिए, जैसे सूचनाएँ न दिखना या ग़लत समय पर आना।

उपयोग का उदाहरण:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat से अनुमतियों की जाँच

अनुमतियों की समस्याएँ डिबग करने के लिए, जैसे अनुमति न मिलना या अस्वीकार होना।

उपयोग का उदाहरण:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat से WebView की जाँच

WebView की समस्याएँ डिबग करने के लिए, जैसे पेज लोड न होना या JavaScript त्रुटि।

उपयोग का उदाहरण:

```
adb logcat -s Chromium WebView
```

उपयोग का उदाहरण (ज़्यादा विवरण):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat से OkHttp की जाँच

OkHttp की समस्याएँ डिबग करने के लिए: नेटवर्क अनुरोध, जवाब और त्रुटियाँ।

उपयोग का उदाहरण:

```
adb logcat -s Okhttp
```

उपयोग का उदाहरण (सभी स्तर):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat से Retrofit की जाँच

Retrofit की समस्याएँ डिबग करने के लिए। चूँकि यह आमतौर पर OkHttp पर चलता है, इसलिए OkHttp के लॉग देखे जाते हैं।

उपयोग का उदाहरण:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat से Glide की जाँच

Glide (इमेज लोड करने वाली लाइब्रेरी) की समस्याएँ डिबग करने के लिए, जैसे इमेज लोड न होना या मेमोरी की समस्याएँ।

उपयोग का उदाहरण:

```
adb logcat -s Glide
```

---

80- Adb Logcat से Picasso की जाँच

Picasso (इमेज लोड करने वाली लाइब्रेरी) की समस्याएँ डिबग करने के लिए।

उपयोग का उदाहरण:

```
adb logcat -s Picasso
```

---

81- Adb Logcat से Room की जाँच

Room (डेटाबेस लाइब्रेरी) की समस्याएँ डिबग करने के लिए: क्वेरी त्रुटियाँ और स्कीमा की समस्याएँ।

उपयोग का उदाहरण:

```
adb logcat -s Room
```

---

82- Adb Logcat से Firebase की जाँच

Crashlytics, Analytics और Messaging जैसे मॉड्यूल में Firebase की समस्याएँ डिबग करने के लिए।

उपयोग का उदाहरण:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat से Crashlytics की जाँच

Firebase Crashlytics की समस्याएँ डिबग करने के लिए।

उपयोग का उदाहरण:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat से Analytics की जाँच

एनालिटिक्स की समस्याएँ डिबग करने के लिए: न भेजी गई घटनाएँ और एक साथ भेजने की समस्याएँ।

उपयोग का उदाहरण:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat के साथ ADB वायरलेस डिबगिंग

वायरलेस डिबगिंग में कभी-कभी adb को रीस्टार्ट करना पड़ता है। पहले USB से जुड़ें, फिर वायरलेस पर जाएँ।

उपयोग का उदाहरण:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat डिवाइस रीस्टार्ट के दौरान

डिवाइस को रीस्टार्ट करता है और लॉग देखता है, ख़ासकर बूट के समय होने वाली त्रुटियों के लिए।

उपयोग का उदाहरण (दो टर्मिनल):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat ऐप रीस्टार्ट के दौरान

एक ऐप को रीस्टार्ट करता है और लॉग देखता है।

उपयोग का उदाहरण:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat ऐप इंस्टॉल के दौरान

एक APK इंस्टॉल करता है और लॉग देखता है।

उपयोग का उदाहरण:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat ऐप अनइंस्टॉल के दौरान

एक ऐप अनइंस्टॉल करता है और लॉग देखता है।

उपयोग का उदाहरण:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat के साथ स्क्रीनशॉट

स्क्रीनशॉट लेता है और त्रुटि के पल का लॉग सहेजता है।

उपयोग का उदाहरण:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat के साथ स्क्रीन रिकॉर्डिंग

स्क्रीन रिकॉर्ड करता है और त्रुटि के पल का लॉग सहेजता है। Android 4.4 से समर्थित।

उपयोग का उदाहरण:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat के साथ Monkey परीक्षण

उपयोगकर्ता के यादृच्छिक इंटरैक्शन की नकल करके ऐप का परीक्षण करता है। लॉग के साथ बहुत प्रभावी।

उपयोग का उदाहरण:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat के साथ UI Automator परीक्षण

इंटरफ़ेस स्वचालन परीक्षणों के लिए, परीक्षण के दौरान लॉग देखते हुए।

उपयोग का उदाहरण:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat के साथ Espresso परीक्षण

Espresso परीक्षणों के लिए, जो इंटरफ़ेस स्वचालन का एक हिस्सा हैं।

उपयोग का उदाहरण:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat के साथ JUnit परीक्षण

यूनिट परीक्षणों के लिए। ये आमतौर पर कंप्यूटर पर चलते हैं, लेकिन डिवाइस पर भी चल सकते हैं।

उपयोग का उदाहरण:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat के साथ StrictMode

किसी ऐप की प्रदर्शन समस्याएँ पकड़ता है। मुख्य थ्रेड पर डिस्क या नेटवर्क तक पहुँचने पर चेतावनी देता है।

उपयोग का उदाहरण:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat के साथ Choreographer

छूटे हुए फ़्रेम पकड़ता है। जब एनिमेशन अटकते हैं तब यहाँ देखा जाता है।

उपयोग का उदाहरण:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat के साथ GPU ओवरड्रॉ

ओवरड्रॉ की समस्याएँ पकड़ता है। ग्राफ़िक्स प्रदर्शन के विश्लेषण में इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat के साथ रेंडर थ्रेड

रेंडर थ्रेड की समस्याएँ पकड़ता है। आधुनिक Android में ड्रॉइंग इसी थ्रेड पर होती है।

उपयोग का उदाहरण:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat के साथ GPU रेंडरिंग

GPU रेंडरिंग की समस्याएँ पकड़ता है। GPU रेंडरिंग प्रोफ़ाइल चालू करने के बाद यहाँ देखा जाता है।

उपयोग का उदाहरण:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat के साथ TextView

टेक्स्ट व्यू की समस्याएँ पकड़ता है, जैसे फ़ॉन्ट की समस्याएँ और माप की त्रुटियाँ।

उपयोग का उदाहरण:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat के साथ RecyclerView

सूची व्यू की समस्याएँ पकड़ता है: स्क्रॉल प्रदर्शन और व्यू का दोबारा उपयोग।

उपयोग का उदाहरण:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat के साथ ViewPager

पेज बदलने में आने वाली समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat के साथ Fragment

Fragment के जीवन-चक्र की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat के साथ Activity

Activity के जीवन-चक्र की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat के साथ सेवाएँ

बैकग्राउंड सेवाओं की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat के साथ WorkManager

बैकग्राउंड कामों के लिए WorkManager की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat के साथ AlarmManager

निर्धारित कामों के लिए AlarmManager की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat के साथ SharedPreferences

SharedPreferences (सरल डेटा संग्रहण) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat के साथ SQLite

SQLite (स्थानीय डेटाबेस) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat के साथ फ़ाइल संचालन

फ़ाइल पढ़ने और लिखने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat के साथ नेटवर्क संचालन

HTTP, TCP और UDP जैसे नेटवर्क कनेक्शन की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat के साथ SSL

SSL प्रमाणपत्र की समस्याएँ पकड़ता है। जब सुरक्षित कनेक्शन विफल होते हैं तब यहाँ देखा जाता है।

उपयोग का उदाहरण:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat के साथ JSON

JSON पार्स करने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat के साथ XML

XML पार्स करने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat के साथ इमेज

इमेज लोड करने और प्रोसेस करने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat के साथ मीडिया

मीडिया चलाने और रिकॉर्ड करने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat के साथ ExoPlayer

ExoPlayer (मीडिया प्लेयर लाइब्रेरी) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat के साथ Chromecast

Chromecast की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat के साथ विज्ञापन

विज्ञापन लाइब्रेरी की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat के साथ बिलिंग

Google Play बिलिंग की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat के साथ मैप

मैप लाइब्रेरी की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat के साथ प्रमाणीकरण

लॉगिन की समस्याएँ पकड़ता है, जैसे Google Sign-In और Firebase Auth।

उपयोग का उदाहरण:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat के साथ सूचनाएँ

सूचनाएँ भेजने और पाने की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat के साथ डीप लिंक

डीप लिंक की समस्याएँ पकड़ता है, जैसे किसी URL से ऐप खोलना।

उपयोग का उदाहरण:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat के साथ Intent

ऐप के बीच आने-जाने में Intent की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat के साथ ब्रॉडकास्ट

सिस्टम और ऐप के बीच ब्रॉडकास्ट संदेशों की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat के साथ कंटेंट प्रोवाइडर

ऐप के बीच डेटा साझा करने वाले कंटेंट प्रोवाइडर की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat के साथ JobScheduler

निर्धारित बैकग्राउंड कामों के लिए JobScheduler की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat के साथ Doze मोड

डिवाइस के सोते समय बैकग्राउंड काम से जुड़ी Doze (नींद) मोड की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat के साथ बैटरी ऑप्टिमाइज़ेशन

बैकग्राउंड प्रतिबंधों से जुड़ी बैटरी ऑप्टिमाइज़ेशन की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat के साथ मेमोरी लीक

मेमोरी लीक पकड़ता है। LeakCanary जैसे टूल के साथ इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat के साथ ANR

ANR (ऐप जवाब नहीं दे रहा) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat के साथ क्रैश

क्रैश पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -b crash
```

---

135- Adb Logcat के साथ नेटिव क्रैश

नेटिव क्रैश (C/C++/Rust) पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat के साथ Tombstone विश्लेषण

नेटिव क्रैश के रिकॉर्ड का विश्लेषण करता है।

उपयोग का उदाहरण:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat के साथ Systrace विश्लेषण

सिस्टम ट्रेसिंग के रिकॉर्ड का विश्लेषण करता है।

उपयोग का उदाहरण:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat के साथ Perfetto विश्लेषण

आधुनिक सिस्टम ट्रेसिंग टूल, जो logcat से ज़्यादा व्यापक है।

उपयोग का उदाहरण:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat के साथ Atrace विश्लेषण

सिस्टम स्तर की ट्रेसिंग के लिए; यह Perfetto का पुराना रूप है।

उपयोग का उदाहरण:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat के साथ Traceview विश्लेषण

मेथड स्तर की प्रोफ़ाइलिंग के लिए।

उपयोग का उदाहरण:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat के साथ मेथड ट्रेसिंग

किसी ख़ास मेथड का पीछा करता है।

उपयोग का उदाहरण:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat के साथ हीप डंप

मेमोरी डंप बनाता है। मेमोरी विश्लेषण में बहुत उपयोगी।

उपयोग का उदाहरण:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat के साथ GC लॉग

गार्बेज कलेक्शन की घटनाएँ देखता है।

उपयोग का उदाहरण:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat के साथ JNI डिबगिंग

JNI (Java Native Interface) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s JNI
```

---

145- Adb Logcat के साथ ART डिबगिंग

ART (Android Runtime) की समस्याएँ पकड़ता है।

उपयोग का उदाहरण:

```
adb logcat -s Art
```

---

146- Adb Logcat के साथ Dalvik डिबगिंग

Dalvik वर्चुअल मशीन की समस्याएँ पकड़ता है। पुराने Android संस्करणों में इस्तेमाल होता है।

उपयोग का उदाहरण:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat और Chatty समस्या

जब logcat बहुत ज़्यादा लॉग लिखता है, तो वह «chatty» चेतावनी देता है और लाइनें छोड़ देता है। इसे फ़िल्टर या बड़े बफ़र से सुलझाया जाता है।

उपयोग का उदाहरण (बफ़र बड़ा करना):

```
adb logcat -G 16M
```

उपयोग का उदाहरण (फ़िल्टर लगाना):

```
adb logcat -s MyApp
```

---

148- Adb Logcat के साथ रंगीन आउटपुट

लंबे लॉग प्रवाह में रंगीन आउटपुट आँखों को कम थकाता है और त्रुटि तुरंत दिखा देता है।

उपयोग का उदाहरण:

```
adb logcat -v color
```

उपयोग का उदाहरण (time के साथ):

```
adb logcat -v time,color
```

---

149- Adb Logcat के साथ Bash स्क्रिप्ट

ऐसी स्क्रिप्ट लिखने के काम आता है जो अपने आप लॉग इकट्ठा करे। इसके ऊपर बग रिपोर्ट करने की प्रणाली बनाई जा सकती है।

उदाहरण स्क्रिप्ट:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat सारांश

जैसा यह सूची दिखाती है, logcat एक अकेला कमांड नहीं बल्कि कमांड का एक समूह है। फ़िल्टर, फ़ॉर्मेट, बफ़र, फ़ाइलों और दूसरे टूल के साथ यह डिबगिंग का सबसे ताक़तवर हथियार बन जाता है। अगर आप जानते हैं कि हर स्थिति में कौन सा मेल इस्तेमाल करना है, तो बग ढूँढने में कुछ ही मिनट लगते हैं। याद रखें: डिबग सत्र का पहला नियम हमेशा एक ही है; पहले adb logcat -c से साफ़ करें, फिर बग दोहराएँ, फिर फ़िल्टर किए गए logcat से देखें।

@@ Android Debug Bridge कनेक्शन
@: ADB वायरलेस कनेक्शन कमांड
1- वायरलेस डिबगिंग चालू करना

पहले डिवाइस पर डेवलपर विकल्प चालू करने होंगे। इसके लिए सेटिंग → फ़ोन के बारे में → बिल्ड नंबर पर 7 बार टैप करें। फिर सेटिंग → सिस्टम → डेवलपर विकल्प → वायरलेस डीबगिंग चालू करें। इस कदम के बिना adb pair और adb connect में से कोई भी कमांड काम नहीं करता।

पाथ का उदाहरण:

```
सेटिंग → फ़ोन के बारे में → बिल्ड नंबर (7 बार टैप करें)
सेटिंग → सिस्टम → डेवलपर विकल्प → वायरलेस डीबगिंग (चालू)
```

---

2- डिवाइस का IP पता और पोर्ट ढूँढना

वायरलेस डीबगिंग चालू होने पर «पेयरिंग कोड से डिवाइस जोड़ें» विकल्प खोलें। वहाँ डिवाइस का IP पता और एक यादृच्छिक पोर्ट नंबर दिखता है, जैसे 192.168.1.42:37123। यह पोर्ट केवल पेयरिंग के लिए है और बाद में बदल जाता है। उसी स्क्रीन पर 6 अंकों का पेयरिंग कोड भी होता है।

स्क्रीन का उदाहरण:

```
IP पता और पोर्ट
192.168.1.42:37123

Wi-Fi पेयरिंग कोड
482915
```

---

3- Adb Pair कमांड

यह कमांड डिवाइस को पेयर करता है। इसे कंप्यूटर पर या Termux में चलाया जाता है। डिवाइस पर दिख रहा IP पता और पेयरिंग पोर्ट लिखें। चलाने पर टर्मिनल Enter pairing code: पूछता है और 6 अंकों के कोड का इंतज़ार करता है। कोड सही हो तो Successfully paired संदेश दिखता है।

उपयोग का उदाहरण:

```
adb pair 192.168.1.42:37123
```

आउटपुट का उदाहरण:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

ग़लत कोड पर यह ऐसे विफल होता है:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Adb Pair को कोड देने के दूसरे तरीके

हर बार हाथ से कोड लिखना ज़रूरी नहीं है। कुछ संस्करण इसे सीधे पैरामीटर के रूप में ले लेते हैं, हालाँकि सुरक्षा के कारण आमतौर पर यह समर्थित नहीं होता। फिर भी आप आज़मा सकते हैं।

उपयोग का उदाहरण (एक लाइन में):

```
adb pair 192.168.1.42:37123 482915
```

उपयोग का उदाहरण (पाइप के साथ):

```
echo "482915" | adb pair 192.168.1.42:37123
```

ये तरीके संस्करण के हिसाब से बदलते हैं। सबसे भरोसेमंद तरीका है कमांड चलाकर कोड हाथ से लिखना।

---

5- Adb Pair में किन बातों का ध्यान रखें

पेयरिंग पोर्ट और कनेक्शन पोर्ट अलग होते हैं। पेयरिंग पोर्ट आमतौर पर यादृच्छिक और बड़ा होता है, कनेक्शन पोर्ट आमतौर पर 5555 होता है। पेयरिंग एक ही बार होती है; उसी डिवाइस से फिर जुड़ने के लिए adb connect काफ़ी है। लेकिन अगर आप डिवाइस को फ़ैक्टरी रीसेट करते हैं या वायरलेस डीबगिंग बंद करते हैं, तो पेयरिंग मिट जाती है और फिर से adb pair करना पड़ता है।

तुलना का उदाहरण:

```
पेयर करना: adb pair 192.168.1.42:37123   → कोड माँगता है, केवल एक बार
जुड़ना:     adb connect 192.168.1.42:5555  → कोड नहीं, हर बार
```

---

6- Adb Connect कमांड

यह कमांड पेयर किए गए डिवाइस से जोड़ता है। वायरलेस डीबगिंग की मुख्य स्क्रीन पर दिख रहा IP पता और पोर्ट लिखें। पोर्ट आमतौर पर 5555 होता है, लेकिन कभी-कभी अलग होता है। कनेक्शन सफल हो तो Connected to संदेश दिखता है।

उपयोग का उदाहरण:

```
adb connect 192.168.1.42:5555
```

आउटपुट का उदाहरण:

```
Connected to 192.168.1.42:5555
```

अगर पहले से जुड़ा हो, तो ऐसे बताता है:

```
Already connected to 192.168.1.42:5555
```

---

7- बिना पोर्ट के Adb Connect

कुछ संस्करणों में पोर्ट न बताने पर डिफ़ॉल्ट रूप से 5555 आज़माया जाता है। लेकिन हर डिवाइस पर ऐसा एक जैसा नहीं होता, इसलिए पोर्ट साफ़-साफ़ लिखना सबसे सुरक्षित है।

उपयोग का उदाहरण (डिफ़ॉल्ट पोर्ट):

```
adb connect 192.168.1.42
```

उपयोग का उदाहरण (स्पष्ट पोर्ट):

```
adb connect 192.168.1.42:5555
```

---

8- कनेक्शन की जाँच

जुड़ने के बाद adb devices कमांड दिखाता है कि डिवाइस सच में जुड़ा है या नहीं। डिवाइस का IP पता सूची में दिखना चाहिए और उसके आगे device लिखा होना चाहिए। अगर offline लिखा हो, तो कनेक्शन बन गया है लेकिन अभी स्थिर नहीं है; थोड़ी देर बाद फिर कोशिश करें।

उपयोग का उदाहरण:

```
adb devices
```

आउटपुट का उदाहरण:

```
List of devices attached
192.168.1.42:5555   device
```

आउटपुट का उदाहरण (नहीं जुड़ा):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Adb Disconnect कमांड

कनेक्शन तोड़ता है। डिवाइस उसी नेटवर्क पर रहे तब भी adb कनेक्शन ख़त्म हो जाता है। फिर जुड़ने के लिए adb connect दोबारा चलाएँ; पेयरिंग की ज़रूरत नहीं।

उपयोग का उदाहरण (एक डिवाइस):

```
adb disconnect 192.168.1.42:5555
```

आउटपुट का उदाहरण:

```
Disconnected 192.168.1.42:5555
```

उपयोग का उदाहरण (सभी डिवाइस):

```
adb disconnect
```

आउटपुट का उदाहरण:

```
Disconnected everything
```

---

10- Adb Tcpip कमांड

USB से जुड़े डिवाइस को वायरलेस मोड में बदलता है। यह कमांड डिवाइस को बताए गए पोर्ट पर सुनने लगाता है, आमतौर पर 5555। यह Android 11 से पहले के उन संस्करणों में ज़्यादा आम तरीका है जिनमें adb pair नहीं है।

उपयोग का उदाहरण:

```
adb tcpip 5555
```

आउटपुट का उदाहरण:

```
Restarting in TCP mode port: 5555
```

फिर USB केबल निकालें और यह कमांड चलाएँ:

```
adb connect 192.168.1.42:5555
```

---

11- Adb Usb कमांड

वायरलेस मोड से USB मोड पर लौटता है, जब आप डिवाइस को फिर से केबल से जोड़ना चाहते हैं।

उपयोग का उदाहरण:

```
adb usb
```

आउटपुट का उदाहरण:

```
Restarting in USB mode
```

---

12- Adb Kill-Server और Start-Server

कभी-कभी adb सर्वर अटक जाता है या कनेक्शन ठुकरा देता है। तब उसे रोककर फिर से शुरू करना पड़ता है। इससे सभी कनेक्शन रीसेट होते हैं और साफ़ शुरुआत मिलती है।

उपयोग का उदाहरण:

```
adb kill-server
adb start-server
```

आउटपुट का उदाहरण:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

फिर से जुड़ें:

```
adb connect 192.168.1.42:5555
```

---

13- एक ही नेटवर्क पर होना

वायरलेस adb के लिए डिवाइस और कंप्यूटर या फ़ोन एक ही Wi-Fi नेटवर्क पर होने चाहिए। अलग-अलग नेटवर्क पर कनेक्शन नहीं होता। गेस्ट नेटवर्क, 5 GHz और 2.4 GHz का बँटवारा, VPN या फ़ायरवॉल कभी-कभी समस्या पैदा करते हैं। आसान हल: दोनों डिवाइस को एक ही राउटर से जोड़ें और VPN बंद करें।

जाँच का उदाहरण:

```
डिवाइस: सेटिंग → Wi-Fi → जुड़ा हुआ नेटवर्क (जैसे Home_Wifi)
Termux: सेटिंग → Wi-Fi → जुड़ा हुआ नेटवर्क (जैसे Home_Wifi)
```

---

14- IP पता हाथ से ढूँढना

डिवाइस का IP पता ढूँढने के लिए सेटिंग → Wi-Fi → जुड़ा हुआ नेटवर्क → नेटवर्क विवरण पर जाएँ। या नीचे दिए कमांड से Termux में देखें। लेकिन IP पता समय के साथ बदल सकता है, इसलिए हर बार वायरलेस डीबगिंग स्क्रीन देखना सबसे भरोसेमंद है।

उपयोग का उदाहरण (Termux):

```
ifconfig wlan0
```

उपयोग का उदाहरण (डिवाइस पर):

```
सेटिंग → Wi-Fi → जुड़ा हुआ नेटवर्क → IP पता
```

---

15- उसी डिवाइस पर Termux से ख़ुद से जुड़ना

अगर Termux उसी डिवाइस पर चल रहा है, तो adb पहले से स्थानीय है, फिर भी आप adb connect से localhost के ज़रिए जुड़ सकते हैं। आमतौर पर 127.0.0.1:5555 इस्तेमाल होता है। कुछ डिवाइस इसे ठुकरा देते हैं; तब डिवाइस का अपना स्थानीय IP इस्तेमाल किया जाता है।

उपयोग का उदाहरण (localhost):

```
adb connect 127.0.0.1:5555
```

उपयोग का उदाहरण (अपना स्थानीय IP):

```
adb connect 192.168.1.42:5555
```

---

16- कई डिवाइस जुड़े होने पर कमांड चलाना

अगर एक साथ कई डिवाइस जुड़े हों, तो हर कमांड में -s पैरामीटर से डिवाइस चुनें। वरना adb error: more than one device के साथ विफल हो जाता है।

उपयोग का उदाहरण:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- जब कनेक्शन टूट जाए

अगर Wi-Fi सिग्नल कमज़ोर हो जाए, डिवाइस स्लीप में चला जाए या नेटवर्क बदल जाए, तो कनेक्शन टूट जाता है। तब डिवाइस adb devices की सूची से ग़ायब हो जाता है या offline दिखता है। हल: adb connect फिर से चलाना। पेयरिंग आमतौर पर मान्य रहती है और adb pair की ज़रूरत नहीं होती।

वापसी का उदाहरण:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- डिवाइस रीस्टार्ट होने के बाद

डिवाइस रीस्टार्ट होने पर वायरलेस डीबगिंग आमतौर पर बंद हो जाती है। कुछ डिवाइस पर यह चालू रहती है, लेकिन पेयरिंग मिट जाती है। डिवाइस चालू होने पर वायरलेस डीबगिंग स्क्रीन पर जाएँ, नया पोर्ट और नया कोड लें, adb pair फिर से चलाएँ और फिर adb connect।

प्रवाह का उदाहरण:

```
डिवाइस रीस्टार्ट हुआ → वायरलेस डीबगिंग चालू करें
नया पोर्ट: 42157, नया कोड: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- पेयरिंग में आम त्रुटियाँ

Failed: Wrong password → कोड ग़लत लिखा गया।
Failed to connect: Connection refused → पोर्ट ग़लत है या डिवाइस सुन नहीं रहा।
Failed: Pairing rejected → डिवाइस ने पेयरिंग ठुकरा दी; कोड नया करें।
Timeout → नेटवर्क में समस्या है; जाँचें कि आप एक ही नेटवर्क पर हैं।

त्रुटि और हल का उदाहरण:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

हल: डिवाइस पर कोड बदल गया है; नया कोड लें और फिर कोशिश करें।

---

20- वायरलेस Adb बंद करना

काम ख़त्म होने पर वायरलेस डीबगिंग बंद करना सुरक्षा के लिए अच्छा है। आप इसे सेटिंग से बंद कर सकते हैं या adb से USB मोड पर जा सकते हैं।

उपयोग का उदाहरण:

```
adb usb
adb disconnect
```

उपयोग का उदाहरण (सेटिंग से):

```
सेटिंग → सिस्टम → डेवलपर विकल्प → वायरलेस डीबगिंग (बंद)
```

---

21- Android संस्करण के हिसाब से अंतर

Android 11 और बाद के: adb pair से पेयरिंग ज़रूरी है और यादृच्छिक पोर्ट इस्तेमाल होता है।
Android 10 और पहले के: adb tcpip 5555 से वायरलेस मोड पर जाया जाता है; पेयरिंग नहीं होती।

तुलना का उदाहरण:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- सुरक्षा संबंधी बातें

वायरलेस डीबगिंग चालू रहने तक उसी नेटवर्क पर कोई बुरी नीयत वाला व्यक्ति डिवाइस तक पहुँचने की कोशिश कर सकता है। इसलिए जिन Wi-Fi नेटवर्क पर आपको भरोसा नहीं, वहाँ यह सुविधा बंद रखें। पेयरिंग कोड सुरक्षा के लिए है, फिर भी सावधान रहना अच्छा है। काम ख़त्म होने पर इसे बंद करें।

सावधानी का उदाहरण:

```
इस्तेमाल न होने पर: सेटिंग → डेवलपर विकल्प → वायरलेस डीबगिंग (बंद)
सुरक्षित नेटवर्क पर: केवल घर या दफ़्तर में चालू करें
```

---

23- पूरा प्रवाह एक जगह

नीचे की स्क्रिप्ट शुरुआत से कनेक्शन तक के सभी कदम दिखाती है। बस IP और पोर्ट को अपने डिवाइस के मानों से बदल दें।

प्रवाह का उदाहरण:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

अपेक्षित परिणाम:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- स्थायी कनेक्शन के लिए शॉर्टकट स्क्रिप्ट

अगर आप अक्सर जुड़ते हैं, तो एक स्क्रिप्ट लिखकर हर बार एक ही कमांड से जुड़ सकते हैं। जैसे Termux में ~/.bashrc फ़ाइल में एक फ़ंक्शन जोड़ सकते हैं।

फ़ंक्शन का उदाहरण:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "डिवाइस पर दिख रहा IP:पोर्ट और कोड लिखें:"
    read -p "पता: " ADRES
    adb pair "$ADRES"
    read -p "कनेक्शन पोर्ट (जैसे 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

उपयोग:

```
wifi_adb
```

---

25- अंतिम टिप्पणी

वायरलेस adb से USB केबल के बिना डिबगिंग होती है, और फ़ोन पर ही Termux में काम करते समय यह ख़ास तौर पर सुविधाजनक है। दो बुनियादी कमांड adb pair और adb connect हैं; पहला एक बार पेयर करता है, दूसरा हर कनेक्शन पर इस्तेमाल होता है। बाकी (adb devices, adb disconnect, adb tcpip, adb kill-server) इन दोनों कमांड का साथ देते हैं और समस्याएँ सुलझाते हैं। एक ही नेटवर्क वाला नियम न भूलें, इस्तेमाल न होने पर बंद करें और कुछ गड़बड़ हो तो सर्वर रीस्टार्ट करें। बस इतना ही।
"#;

const INDONESIAN: &str = r#"@@ Perintah Logcat
1- Adb Logcat

Mengirim semua log perangkat secara langsung ke terminal. Sejak perintah dijalankan, setiap baris log yang dibuat oleh sistem, aplikasi, layanan, dan kernel muncul di layar. Untuk menghentikannya, tekan Ctrl + C di keyboard. Perintah ini adalah dasar dari debugging; tetapi tanpa filter, ia bisa mengalirkan ratusan baris per detik sehingga yang Anda cari sulit ditemukan.

Contoh penggunaan:

```
adb logcat
```

Contoh keluaran:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Menampilkan log yang sudah terkumpul lalu perintah selesai. Alih-alih aliran terus-menerus, ia mengambil potret saat itu. Gunakan saat menyiapkan laporan bug, menyimpan keadaan setelah crash, atau ketika ingin mengambil log sekali saja dari skrip.

Contoh penggunaan:

```
adb logcat -d
```

Contoh penggunaan (simpan ke berkas):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Perintah ini sering dipakai dalam skrip yang otomatis mengirim laporan ke server, karena ia selesai sendiri dan tidak pernah membuat skrip macet.

---

3- Adb Logcat -C

Mengosongkan semua buffer log dan menghapus semua yang terkumpul sampai saat itu. Jika Anda menjalankannya sebelum sesi debugging baru, Anda hanya akan melihat log yang muncul setelahnya dan kebisingan lama tidak akan membingungkan Anda. Saat mencoba mereproduksi bug, biasanya ini langkah pertama.

Contoh penggunaan:

```
adb logcat -c
adb logcat
```

Dengan menjalankan dua perintah di atas secara berurutan, Anda mendapat aliran log yang benar-benar bersih. Hanya yang terjadi setelah pengosongan yang muncul.

---

4- Adb Logcat -G

Menampilkan ukuran buffer log saat ini dan seberapa penuh buffer itu. Mencantumkan baris demi baris isi setiap buffer, kapasitas total, dan jumlah yang terpakai. Jika log mengalir terlalu cepat dan entri lama dibuang (ini disebut «chatty»), lihat di sini dulu.

Contoh penggunaan:

```
adb logcat -g
```

Contoh keluaran:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Ukuran

Mengubah ukuran buffer log. Misalnya, buffer 256 KiB bisa dinaikkan menjadi 16 MiB. Dengan begitu lebih banyak log disimpan dan entri lama dibuang lebih lambat. Sangat berguna dalam pengujian panjang atau saat men-debug aplikasi yang sangat bising. Pengaturan kembali seperti semula saat perangkat dimulai ulang.

Contoh penggunaan:

```
adb logcat -G 16M
```

Contoh penggunaan (untuk mengecilkan):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Berkas

Menulis log ke berkas yang ditentukan alih-alih menampilkannya di terminal. Terminal tetap bersih dan log tersimpan. Jalur berkas berada di perangkat, bukan di komputer. Di terminal ponsel (Termux) biasanya ditulis ke /Sdcard/.

Contoh penggunaan:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Contoh penggunaan (menambahkan ke berkas):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

Pada contoh di atas, berkas baru dibuka ketika berkas mencapai 4 MiB, dan total 10 berkas disimpan.

---

7- Adb Logcat -B Main

Hanya memantau buffer utama aplikasi. Di sini ada log yang ditulis aplikasi dengan panggilan seperti Log.d, Log.i, dan Log.e. Buffer ini sudah dipantau secara bawaan, tetapi ditulis secara eksplisit saat Anda hanya menginginkan buffer ini. Inilah tempat yang paling sering dilihat saat mengembangkan aplikasi.

Contoh penggunaan:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Memantau log layanan sistem. Di sini ada log layanan seperti ActivityManager, WindowManager, PackageManager, dan PowerManager. Berguna untuk melihat peristiwa seperti aplikasi dimulai, izin diberikan, dan pengikatan ke layanan. Biasanya dipantau bersama main.

Contoh penggunaan:

```
adb logcat -b system
```

Contoh keluaran:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Hanya memantau log crash. Di sini ada blok FATAL EXCEPTION yang ditulis saat aplikasi ditutup paksa. Saat men-debug crash, melihat buffer ini lebih dulu adalah jalan tercepat, karena stack trace ada tepat di sini.

Contoh penggunaan:

```
adb logcat -b crash
```

Contoh keluaran:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Memantau log radio, telepon, sinyal, dan koneksi data. Berguna saat menangani kartu SIM, registrasi jaringan, koneksi 4G/5G, SMS, atau panggilan. Biasanya tidak diperlukan dalam pengembangan aplikasi sehari-hari.

Contoh penggunaan:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Memantau log biner peristiwa sistem. Buffer ini tidak berisi teks biasa, melainkan catatan singkat berkode yang ditinggalkan panggilan Log.event. Catatan itu muncul dengan tag seperti am_proc_start dan am_activity_launch. Dipakai dalam analisis kinerja.

Contoh penggunaan:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Memantau log kernel. Menampilkan pesan yang ditulis kernel Linux, yang biasanya mirip keluaran perintah dmesg. Di sini ada galat driver, masalah memori, serta informasi suhu dan manajemen daya. Mungkin memerlukan root.

Contoh penggunaan:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Memantau semua buffer sekaligus. main, system, crash, radio, events, dan kernel mengalir bercampur. Berguna saat ingin melihat semuanya, tetapi keluarannya jadi sangat padat, jadi biasanya digabung dengan filter.

Contoh penggunaan:

```
adb logcat -b all
```

Contoh penggunaan (dengan filter):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Beberapa Buffer

Memantau beberapa buffer yang dipisahkan koma secara bersamaan, misalnya untuk melihat log aplikasi dan crash bersama-sama. Dengan memilih hanya saluran yang diperlukan, Anda mengurangi keluaran.

Contoh penggunaan:

```
adb logcat -b main,crash
```

Contoh penggunaan (tiga saluran):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

Format keluaran singkat bawaan. Setiap baris hanya diawali prioritas dan tag. Bagus untuk melihat sekilas, tetapi tanpa cap waktu sulit mengurutkan peristiwa.

Contoh penggunaan:

```
adb logcat -v brief
```

Contoh keluaran:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Menambahkan tanggal dan waktu. Jauh lebih berguna daripada format singkat, karena Anda bisa melihat kapan setiap peristiwa terjadi. Setelah detik juga ditampilkan milidetik.

Contoh penggunaan:

```
adb logcat -v time
```

Contoh keluaran:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

Format keluaran yang paling disarankan. Berisi tanggal, waktu, prioritas, tag, PID (pengenal proses), dan TID (pengenal thread). Pada aplikasi multithread, format ini sangat penting untuk tahu thread mana melakukan apa. Ini juga format yang dipakai Android Studio secara bawaan.

Contoh penggunaan:

```
adb logcat -v threadtime
```

Contoh keluaran:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Memberikan keluaran rinci dengan semua kolom metadata dan satu baris kosong di antara entri. Dalam sesi panjang, setiap log ditampilkan sebagai satu blok, tetapi memakan tempat.

Contoh penggunaan:

```
adb logcat -v long
```

Contoh keluaran:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Mewarnai log sesuai prioritas. Galat berwarna merah, peringatan jingga, info hijau, dan debug biru. Dalam aliran panjang, mata langsung tertuju ke galat. Terminal harus mendukung warna.

Contoh penggunaan:

```
adb logcat -v color
```

Contoh penggunaan (bersama time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Menambahkan tahun ke format waktu. Dipakai pada log panjang atau arsip yang mencakup lebih dari satu tahun. Biasanya tidak diperlukan dalam pengembangan sehari-hari.

Contoh penggunaan:

```
adb logcat -v year
```

Contoh keluaran:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Menampilkan pengenal pengguna (UID) aplikasi yang membuat log. Berguna saat beberapa aplikasi memakai tag yang sama. Ini adalah pengubah, jadi digabung dengan format lain.

Contoh penggunaan:

```
adb logcat -v uid
```

Contoh keluaran:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filter Tag Dan Prioritas

Menampilkan log dari tag tertentu mulai dari prioritas tertentu. Prioritasnya: V (Verbose) paling rendah, D (Debug), I (Info), W (Warning), E (Error), F (Fatal), dan S (Silent) paling tinggi. Cara memfilter ini adalah fitur logcat yang paling kuat.

Contoh penggunaan (hanya galat):

```
adb logcat *:E
```

Contoh penggunaan (peringatan ke atas):

```
adb logcat *:W
```

Contoh penggunaan (daftar izin):

```
adb logcat ActivityManager:I MyApp:D *:S
```

Contoh terakhir menampilkan ActivityManager mulai dari info dan MyApp mulai dari debug, lalu membungkam semua tag lain. Di beberapa shell, karakter * bersifat khusus, jadi harus diapit tanda kutip:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Tag

Jalan pintas untuk filter senyap. Hanya menampilkan tag yang ditentukan dan membungkam semua yang lain. Cara paling praktis untuk fokus pada tag Anda sendiri saat mengembangkan.

Contoh penggunaan:

```
adb logcat -s MyApp
```

Contoh penggunaan (beberapa tag):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

Pada contoh di atas hanya log dari tiga tag yang lolos; sisanya senyap.

---

24- Adb Logcat -S Dengan Prioritas

Dipakai untuk menentukan tag dan prioritas sekaligus, misalnya untuk melihat hanya peringatan ke atas dari tag MyApp.

Contoh penggunaan:

```
adb logcat MyApp:W *:S
```

Perintah ini hanya menampilkan log warning, error, dan fatal dengan tag MyApp.

---

25- Adb Logcat Filter --Pid

Hanya menampilkan log dari pengenal proses (PID) tertentu. Untuk mengetahui PID sebuah aplikasi, gunakan dulu adb shell pidof, lalu filter dengan PID itu. Sempurna untuk memisahkan semua log dari satu sesi.

Contoh penggunaan (mengambil PID):

```
adb shell pidof com.example.app
```

Contoh keluaran:

```
5678
```

Contoh penggunaan (memfilter):

```
adb logcat --pid=5678
```

Contoh penggunaan (dalam satu baris):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Waktu

Menampilkan log mulai dari cap waktu yang ditentukan, artinya «ambil yang datang setelah saat ini». Ditulis dengan T besar; t kecil berarti hal lain.

Contoh penggunaan:

```
adb logcat -T "09-21 14:00:00.000"
```

Contoh penggunaan (mulai hari ini):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Jumlah Baris

Dengan t kecil, menampilkan sejumlah baris terbaru lalu perintah selesai. Bekerja seperti -d, tetapi hanya memberi N baris terakhir. Ideal untuk melihat sekilas.

Contoh penggunaan (100 baris terakhir):

```
adb logcat -t 100
```

Contoh penggunaan (500 baris terakhir dan filter):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Mencari isi dengan ekspresi reguler (regex) dan menampilkan baris yang cocok. Sangat kuat untuk pencarian teks cepat; terutama saat mencari beberapa kata sekaligus, lebih cepat daripada memfilter.

Contoh penggunaan (satu kata):

```
adb logcat -e "Login"
```

Contoh penggunaan (beberapa kata):

```
adb logcat -e "Login|Error|Crash"
```

Contoh penggunaan (dengan nama paket):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Keluaran Biner

Menulis log dalam format biner. Manusia tidak bisa membacanya, tetapi logcat lain bisa membacanya kembali atau alat lain bisa mengolahnya. Dipakai dalam sistem analisis otomatis.

Contoh penggunaan:

```
adb logcat -B > Ham_Log.Bin
```

Nanti berkas itu dibaca seperti ini:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Statistik

Menampilkan statistik log: berapa banyak log yang dibuat setiap tag dan tag mana yang memicu peringatan «chatty». Jika Anda merasa sebuah aplikasi membanjiri log, perintah ini memastikannya.

Contoh penggunaan:

```
adb logcat -S
```

Contoh keluaran:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R Dan -N Bersama

Untuk log bergilir. -r menentukan ukuran setiap berkas dan -n berapa banyak berkas yang disimpan. Berguna saat aplikasi akan berjalan lama dan Anda ingin mencatat terus-menerus. Berkas lama dihapus dan berkas baru dibuka.

Contoh penggunaan:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

Pada contoh di atas, setiap kali sebuah berkas mencapai 4 MiB, berkas baru dibuka dan total 10 berkas disimpan. Dengan begitu disk tidak pernah penuh tanpa batas.

---

32- Adb Logcat -P Skrip Filter

Menentukan skrip filter khusus untuk logcat. Ini format filter khas Android dan biasanya dipakai pengembang sistem. Pengembang aplikasi jarang memakainya.

Contoh penggunaan:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Cap Waktu

Opsi yang pada beberapa format menambahkan cap waktu di awal setiap baris. Perilakunya berbeda antarversi Android. Biasanya melakukan hal yang sama dengan -v time, tetapi dengan cara lain.

Contoh penggunaan:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Menampilkan setiap baris dalam format printf, yaitu prioritas (PID) tag: pesan. Dipakai demi kompatibilitas dengan alat lama, tetapi jarang diperlukan di Android modern.

Contoh penggunaan:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Hanya menampilkan pesan mentah tanpa metadata. Hanya isi log yang ditulis, tanpa tag, prioritas, atau cap waktu. Berguna untuk meneruskan keluaran ke program lain.

Contoh penggunaan:

```
adb logcat -v raw
```

Contoh keluaran:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Hanya menulis tag di awal setiap baris, lalu sisanya. Prioritas dan PID disembunyikan. Praktis saat Anda fokus pada satu tag.

Contoh penggunaan:

```
adb logcat -v tag
```

Contoh keluaran:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Menulis PID dulu lalu tag di awal setiap baris. Praktis untuk analisis per proses; Anda cepat melihat proses mana menulis apa.

Contoh penggunaan:

```
adb logcat -v process
```

Contoh keluaran:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Hanya menulis pengenal thread (TID) di awal setiap baris, lalu sisanya. Dalam analisis multithread, menunjukkan thread mana yang aktif. Format yang singkat.

Contoh penggunaan:

```
adb logcat -v thread
```

Contoh keluaran:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D Dan -T Bersama

Mengambil log dari rentang waktu tertentu, misalnya satu jam terakhir, untuk disimpan ke berkas.

Contoh penggunaan:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Contoh penggunaan (ke berkas):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C Dan -D Berurutan

Alur yang mengosongkan dulu lalu mengumpulkan. Misalnya mengosongkan sebelum pengujian dimulai dan mengumpulkan log di akhir pengujian.

Contoh penggunaan:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Disimpan Ke Berkas Dan Dipindahkan Ke Komputer

Praktis untuk mencatat lama di terminal ponsel. Pertama ditulis ke berkas di perangkat, lalu dipindahkan ke komputer dengan adb pull.

Contoh penggunaan (simpan di perangkat):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Contoh penggunaan (pindahkan ke komputer):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Dengan Grep

Dipakai di terminal ponsel (Termux) untuk mencari teks dengan cepat lewat grep. Bisa lebih fleksibel daripada filter regex, karena semua opsi grep tersedia.

Contoh penggunaan:

```
adb logcat | grep "Hata"
```

Contoh penggunaan (tanpa membedakan huruf besar dan kecil):

```
adb logcat | grep -I "error"
```

Contoh penggunaan (dengan nomor baris):

```
adb logcat | grep -N "Exception"
```

Contoh penggunaan (beberapa kata):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Dengan Tail

Memangkas aliran langsung menjadi beberapa baris terakhirnya, saat Anda hanya ingin melihat bagian akhir, bukan seluruh log.

Contoh penggunaan:

```
adb logcat | tail -100
```

Contoh penggunaan (mengikuti terus):

```
adb logcat | tail -F
```

---

44- Adb Logcat Dengan Head

Menampilkan baris-baris pertama. Bagus untuk melihat sekilas log yang panjang.

Contoh penggunaan:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Dengan Wc

Menampilkan jumlah total baris, misalnya untuk mengukur berapa banyak log yang dihasilkan sebuah pengujian.

Contoh penggunaan:

```
adb logcat -d | wc -l
```

Contoh penggunaan (hanya menghitung galat):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Dengan Sort

Mengurutkan log menurut abjad atau angka. Sebagian besar dipakai saat analisis.

Contoh penggunaan:

```
adb logcat -d | sort
```

Contoh penggunaan (urutan terbalik):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Dengan Uniq

Menghapus baris yang berulang. Jika sebuah aplikasi terus menulis galat yang sama, perintah ini menyederhanakan keluaran.

Contoh penggunaan:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

Contoh di atas menghitung dan menampilkan 20 baris yang paling sering berulang. Sangat berguna dalam debugging.

---

48- Adb Logcat Dengan Awk

Mengambil kolom tertentu atau mengolah teks, misalnya kolom PID pada format threadtime.

Contoh penggunaan (hanya kolom PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Contoh penggunaan (hanya tag dan pesan):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Dengan Xargs

Menjalankan perintah untuk setiap baris, misalnya mengirim notifikasi untuk setiap baris galat. Sebagian besar dipakai dalam otomatisasi.

Contoh penggunaan:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Dengan Sed

Mengganti teks. Ideal untuk mengubah pola tertentu dalam keluaran.

Contoh penggunaan:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

Contoh di atas menulis Uygulama di tempat setiap MyApp.

---

51- Adb Logcat Dengan Tee

Menampilkan di layar dan menulis ke berkas pada saat yang sama. Pengamatan langsung dan pencatatan berjalan bersamaan.

Contoh penggunaan:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Dengan Watch

Mengambil log pada selang waktu tetap, misalnya ringkasan setiap 5 detik.

Contoh penggunaan:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Dengan Timeout

Menghentikan logcat setelah waktu yang ditentukan. Sangat berguna dalam pengujian otomatis.

Contoh penggunaan:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Dengan Alias

Memperpendek perintah yang sering Anda pakai. Di Termux ditambahkan ke berkas .bashrc atau .zshrc.

Contoh penggunaan (mendefinisikan alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Setelah itu cukup panggil seperti ini:

```
lce
```

---

55- Adb Logcat Dengan Beberapa Perangkat

Menunjuk perangkat tertentu saat beberapa perangkat terhubung. Pertama tampilkan daftar perangkat dengan adb devices, lalu pilih satu dengan -s.

Contoh penggunaan:

```
adb devices
```

Contoh keluaran:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Contoh penggunaan (perangkat tertentu):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Dengan Debugging Nirkabel

Untuk debugging nirkabel. Saat perangkat berada di jaringan Wi-Fi yang sama, koneksi dibuat tanpa kabel. Didukung sejak Android 11.

Contoh penggunaan (aktifkan debugging nirkabel di perangkat, lalu):

```
adb pair 192.168.1.42:37123
```

Contoh penggunaan (koneksi):

```
adb connect 192.168.1.42:5555
```

Contoh penggunaan (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Dengan Bugreport

Membuat laporan rinci tentang seluruh sistem. Selain logcat, banyak alat seperti dumpsys, dmesg, dan top dijalankan, dan semuanya masuk ke satu zip. Cara debugging paling lengkap.

Contoh penggunaan:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Contoh penggunaan (pindahkan ke komputer):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Dengan Dumpsys

Menampilkan, di samping logcat, keadaan terkini layanan sistem, misalnya apa yang sedang dilakukan ActivityManager saat itu.

Contoh penggunaan:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Dengan Jejak ANR

Mengambil stack trace saat aplikasi tidak merespons (ANR). Catatan ANR ada di berkas /Data/Anr/Traces.Txt.

Contoh penggunaan:

```
adb shell cat /Data/Anr/Traces.Txt
```

Contoh penggunaan (memerlukan root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Dengan Tombstone

Untuk crash native. Di folder /Data/Tombstones/ dibuat satu berkas untuk setiap crash. Di sinilah dilihat saat kode C++ atau Rust mengalami galat.

Contoh penggunaan:

```
adb shell ls /Data/Tombstones/
```

Contoh penggunaan (tombstone terbaru):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Dengan Systrace

Untuk pelacakan kinerja seluruh sistem. Mengumpulkan data jejak tingkat kernel, bukan hanya log. Dipakai bersama Perfetto.

Contoh penggunaan:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Dengan Layout Inspector

Menganalisis tata letak aplikasi di layar. Dipakai dengan Layout Inspector, tetapi log terkait juga bisa diikuti dengan logcat.

Contoh penggunaan:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Dengan Memory Profiler

Memantau penggunaan memori. Memory Profiler biasanya ada di Android Studio, tetapi log terkait memori juga terlihat di logcat.

Contoh penggunaan:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Dengan CPU Profiler

Memantau penggunaan CPU. Untuk pengukuran sederhana, perintah top juga cukup.

Contoh penggunaan:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Dengan Network Profiler

Memantau lalu lintas jaringan. Pustaka seperti OkHttp dan Retrofit menulis ke logcat, tetapi analisis lengkap memerlukan Network Profiler.

Contoh penggunaan:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Dengan GPU Profiler

Memantau penggunaan GPU. Untuk pengukuran sederhana, perintah gfxinfo juga cukup.

Contoh penggunaan:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Dengan Battery Historian

Memantau konsumsi baterai. Bisa juga dilihat dengan dumpsys battery.

Contoh penggunaan:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Untuk Mendiagnosis Wi-Fi

Dipakai untuk men-debug masalah Wi-Fi, seperti koneksi terputus atau sinyal lemah.

Contoh penggunaan:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Untuk Mendiagnosis Bluetooth

Dipakai untuk men-debug masalah Bluetooth, seperti gagal menyandingkan atau koneksi terputus.

Contoh penggunaan:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Untuk Mendiagnosis Sensor

Dipakai untuk men-debug masalah sensor seperti akselerometer, giroskop, dan magnetometer.

Contoh penggunaan:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Untuk Mendiagnosis Kamera

Dipakai untuk men-debug masalah kamera, seperti mengambil foto atau merekam video.

Contoh penggunaan:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Untuk Mendiagnosis Audio

Dipakai untuk men-debug masalah audio, seperti tidak ada suara atau mikrofon tidak berfungsi.

Contoh penggunaan:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Untuk Mendiagnosis Lokasi

Dipakai untuk men-debug masalah lokasi, seperti tidak ada sinyal GPS atau lokasi tidak diperbarui.

Contoh penggunaan:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Untuk Mendiagnosis Notifikasi

Dipakai untuk men-debug masalah notifikasi, seperti notifikasi tidak muncul atau datang pada waktu yang salah.

Contoh penggunaan:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Untuk Mendiagnosis Izin

Dipakai untuk men-debug masalah izin, seperti izin tidak diberikan atau ditolak.

Contoh penggunaan:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Untuk Mendiagnosis WebView

Dipakai untuk men-debug masalah WebView, seperti halaman tidak termuat atau galat JavaScript.

Contoh penggunaan:

```
adb logcat -s Chromium WebView
```

Contoh penggunaan (lebih rinci):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Untuk Mendiagnosis OkHttp

Dipakai untuk men-debug masalah OkHttp: permintaan jaringan, respons, dan galat.

Contoh penggunaan:

```
adb logcat -s Okhttp
```

Contoh penggunaan (semua tingkat):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Untuk Mendiagnosis Retrofit

Dipakai untuk men-debug masalah Retrofit. Karena biasanya berjalan di atas OkHttp, yang dilihat adalah log OkHttp.

Contoh penggunaan:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Untuk Mendiagnosis Glide

Dipakai untuk men-debug masalah Glide (pustaka pemuat gambar), seperti gambar tidak termuat atau masalah memori.

Contoh penggunaan:

```
adb logcat -s Glide
```

---

80- Adb Logcat Untuk Mendiagnosis Picasso

Dipakai untuk men-debug masalah Picasso (pustaka pemuat gambar).

Contoh penggunaan:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Untuk Mendiagnosis Room

Dipakai untuk men-debug masalah Room (pustaka basis data): galat kueri dan masalah skema.

Contoh penggunaan:

```
adb logcat -s Room
```

---

82- Adb Logcat Untuk Mendiagnosis Firebase

Dipakai untuk men-debug masalah Firebase pada modul seperti Crashlytics, Analytics, dan Messaging.

Contoh penggunaan:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Untuk Mendiagnosis Crashlytics

Dipakai untuk men-debug masalah Firebase Crashlytics.

Contoh penggunaan:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Untuk Mendiagnosis Analytics

Dipakai untuk men-debug masalah analitik: peristiwa yang tidak terkirim dan masalah pengiriman massal.

Contoh penggunaan:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Dengan Debugging ADB Nirkabel

Debugging nirkabel terkadang memerlukan adb dimulai ulang. Hubungkan dulu lewat USB, lalu beralih ke nirkabel.

Contoh penggunaan:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Saat Perangkat Dimulai Ulang

Memulai ulang perangkat dan memantau log, terutama untuk galat yang terjadi saat booting.

Contoh penggunaan (dua terminal):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Saat Aplikasi Dimulai Ulang

Memulai ulang satu aplikasi dan memantau log.

Contoh penggunaan:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Saat Aplikasi Dipasang

Memasang APK dan memantau log.

Contoh penggunaan:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Saat Aplikasi Dicopot

Mencopot aplikasi dan memantau log.

Contoh penggunaan:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Dengan Tangkapan Layar

Mengambil tangkapan layar dan menyimpan log pada saat galat terjadi.

Contoh penggunaan:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Dengan Perekaman Layar

Merekam layar dan menyimpan log pada saat galat terjadi. Didukung sejak Android 4.4.

Contoh penggunaan:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Dengan Uji Monkey

Menguji aplikasi dengan menyimulasikan interaksi pengguna secara acak. Sangat efektif bersama log.

Contoh penggunaan:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Dengan Uji UI Automator

Untuk pengujian otomatisasi antarmuka, sambil memantau log selama pengujian.

Contoh penggunaan:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Dengan Uji Espresso

Untuk pengujian Espresso, bagian dari otomatisasi antarmuka.

Contoh penggunaan:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Dengan Uji JUnit

Untuk pengujian unit. Biasanya berjalan di komputer, tetapi juga bisa berjalan di perangkat.

Contoh penggunaan:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Dengan StrictMode

Menemukan masalah kinerja aplikasi. Memberi peringatan saat disk atau jaringan diakses di thread utama.

Contoh penggunaan:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Dengan Choreographer

Menemukan frame yang terlewat. Di sinilah dilihat saat animasi tersendat.

Contoh penggunaan:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Dengan Overdraw GPU

Menemukan masalah overdraw. Dipakai dalam analisis kinerja grafis.

Contoh penggunaan:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Dengan Thread Render

Menemukan masalah thread render. Di Android modern, penggambaran terjadi di thread ini.

Contoh penggunaan:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Dengan Rendering GPU

Menemukan masalah rendering GPU. Di sinilah dilihat setelah profil rendering GPU diaktifkan.

Contoh penggunaan:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Dengan TextView

Menemukan masalah tampilan teks, seperti masalah font dan galat pengukuran.

Contoh penggunaan:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Dengan RecyclerView

Menemukan masalah tampilan daftar: kinerja gulir dan daur ulang tampilan.

Contoh penggunaan:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Dengan ViewPager

Menemukan masalah pada perpindahan halaman.

Contoh penggunaan:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Dengan Fragment

Menemukan masalah siklus hidup fragment.

Contoh penggunaan:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Dengan Activity

Menemukan masalah siklus hidup activity.

Contoh penggunaan:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Dengan Layanan

Menemukan masalah layanan latar belakang.

Contoh penggunaan:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Dengan WorkManager

Menemukan masalah WorkManager, untuk tugas latar belakang.

Contoh penggunaan:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Dengan AlarmManager

Menemukan masalah AlarmManager, untuk tugas terjadwal.

Contoh penggunaan:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Dengan SharedPreferences

Menemukan masalah SharedPreferences (penyimpanan data sederhana).

Contoh penggunaan:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Dengan SQLite

Menemukan masalah SQLite (basis data lokal).

Contoh penggunaan:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Dengan Operasi Berkas

Menemukan masalah membaca dan menulis berkas.

Contoh penggunaan:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Dengan Operasi Jaringan

Menemukan masalah koneksi jaringan, seperti HTTP, TCP, dan UDP.

Contoh penggunaan:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Dengan SSL

Menemukan masalah sertifikat SSL. Di sinilah dilihat saat koneksi aman gagal.

Contoh penggunaan:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Dengan JSON

Menemukan masalah penguraian JSON.

Contoh penggunaan:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Dengan XML

Menemukan masalah penguraian XML.

Contoh penggunaan:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Dengan Gambar

Menemukan masalah pemuatan dan pengolahan gambar.

Contoh penggunaan:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Dengan Media

Menemukan masalah pemutaran dan perekaman media.

Contoh penggunaan:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Dengan ExoPlayer

Menemukan masalah ExoPlayer (pustaka pemutar media).

Contoh penggunaan:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Dengan Chromecast

Menemukan masalah Chromecast.

Contoh penggunaan:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Dengan Iklan

Menemukan masalah pustaka iklan.

Contoh penggunaan:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Dengan Penagihan

Menemukan masalah penagihan Google Play.

Contoh penggunaan:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Dengan Peta

Menemukan masalah pustaka peta.

Contoh penggunaan:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Dengan Autentikasi

Menemukan masalah masuk akun, seperti Google Sign-In dan Firebase Auth.

Contoh penggunaan:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Dengan Notifikasi

Menemukan masalah pengiriman dan penerimaan notifikasi.

Contoh penggunaan:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Dengan Deep Link

Menemukan masalah deep link, seperti membuka aplikasi dari URL.

Contoh penggunaan:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Dengan Intent

Menemukan masalah intent saat berpindah antaraplikasi.

Contoh penggunaan:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Dengan Broadcast

Menemukan masalah pesan broadcast antara sistem dan aplikasi.

Contoh penggunaan:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Dengan Content Provider

Menemukan masalah content provider, untuk berbagi data antaraplikasi.

Contoh penggunaan:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Dengan JobScheduler

Menemukan masalah JobScheduler, untuk tugas latar belakang terjadwal.

Contoh penggunaan:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Dengan Mode Doze

Menemukan masalah mode Doze (tidur), untuk pekerjaan latar belakang saat perangkat tidur.

Contoh penggunaan:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Dengan Optimalisasi Baterai

Menemukan masalah optimalisasi baterai, untuk pembatasan latar belakang.

Contoh penggunaan:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Dengan Kebocoran Memori

Menemukan kebocoran memori. Dipakai bersama alat seperti LeakCanary.

Contoh penggunaan:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Dengan ANR

Menemukan masalah ANR (aplikasi tidak merespons).

Contoh penggunaan:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Dengan Crash

Menemukan crash.

Contoh penggunaan:

```
adb logcat -b crash
```

---

135- Adb Logcat Dengan Crash Native

Menemukan crash native (C/C++/Rust).

Contoh penggunaan:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Dengan Analisis Tombstone

Menganalisis catatan crash native.

Contoh penggunaan:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Dengan Analisis Systrace

Menganalisis catatan pelacakan sistem.

Contoh penggunaan:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Dengan Analisis Perfetto

Alat pelacakan sistem modern, lebih lengkap daripada logcat.

Contoh penggunaan:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Dengan Analisis Atrace

Untuk pelacakan tingkat sistem; bentuk lama dari Perfetto.

Contoh penggunaan:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Dengan Analisis Traceview

Untuk profiling tingkat metode.

Contoh penggunaan:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Dengan Pelacakan Metode

Mengikuti metode tertentu.

Contoh penggunaan:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Dengan Heap Dump

Membuat dump memori. Sangat berguna dalam analisis memori.

Contoh penggunaan:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Dengan Log GC

Memantau peristiwa pengumpulan sampah (garbage collection).

Contoh penggunaan:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Dengan Debugging JNI

Menemukan masalah JNI (Java Native Interface).

Contoh penggunaan:

```
adb logcat -s JNI
```

---

145- Adb Logcat Dengan Debugging ART

Menemukan masalah ART (Android Runtime).

Contoh penggunaan:

```
adb logcat -s Art
```

---

146- Adb Logcat Dengan Debugging Dalvik

Menemukan masalah mesin virtual Dalvik. Dipakai pada versi Android yang lebih lama.

Contoh penggunaan:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat Dan Masalah Chatty

Saat logcat menulis terlalu banyak log, ia memberi peringatan «chatty» dan melewatkan baris. Diatasi dengan filter atau buffer yang lebih besar.

Contoh penggunaan (memperbesar buffer):

```
adb logcat -G 16M
```

Contoh penggunaan (memakai filter):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Dengan Keluaran Berwarna

Dalam aliran log yang panjang, keluaran berwarna membuat mata tidak cepat lelah dan langsung menunjukkan galat.

Contoh penggunaan:

```
adb logcat -v color
```

Contoh penggunaan (dengan time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Dengan Skrip Bash

Dipakai untuk menulis skrip yang mengumpulkan log secara otomatis. Di atasnya bisa dibangun sistem pelaporan bug.

Contoh skrip:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Ringkasan Adb Logcat

Seperti ditunjukkan daftar ini, logcat bukan satu perintah, melainkan sekumpulan perintah. Bersama filter, format, buffer, berkas, dan alat lain, ia menjadi senjata debugging paling kuat. Jika Anda tahu kombinasi mana yang dipakai di setiap situasi, menemukan bug hanya butuh beberapa menit. Ingat: aturan pertama sesi debugging selalu sama; kosongkan dulu dengan adb logcat -c, lalu reproduksi bug, lalu amati dengan logcat yang difilter.

@@ Koneksi Android Debug Bridge
@: Perintah Koneksi ADB Nirkabel
1- Mengaktifkan Debugging Nirkabel

Pertama, opsi developer harus diaktifkan di perangkat. Untuk itu, ketuk 7 kali Setelan → Tentang ponsel → Nomor build. Lalu aktifkan Setelan → Sistem → Opsi developer → Debugging nirkabel. Tanpa langkah ini, perintah adb pair dan adb connect tidak akan berfungsi.

Contoh jalur:

```
Setelan → Tentang ponsel → Nomor build (ketuk 7 kali)
Setelan → Sistem → Opsi developer → Debugging nirkabel (aktif)
```

---

2- Menemukan Alamat IP Dan Port Perangkat

Dengan debugging nirkabel aktif, buka opsi «Sambungkan perangkat dengan kode penyambungan». Di sana muncul alamat IP perangkat dan nomor port acak, misalnya 192.168.1.42:37123. Port ini hanya untuk penyambungan dan kemudian berubah. Di layar yang sama juga ada kode penyambungan 6 digit.

Contoh tampilan:

```
Alamat IP dan port
192.168.1.42:37123

Kode penyambungan Wi-Fi
482915
```

---

3- Perintah Adb Pair

Perintah ini menyambungkan perangkat. Dijalankan di komputer atau di Termux. Tulis alamat IP dan port penyambungan yang ditampilkan di perangkat. Saat dijalankan, terminal bertanya Enter pairing code: dan menunggu kode 6 digit. Jika kodenya benar, muncul pesan Successfully paired.

Contoh penggunaan:

```
adb pair 192.168.1.42:37123
```

Contoh keluaran:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Dengan kode yang salah, perintah gagal seperti ini:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Cara Lain Memberikan Kode Ke Adb Pair

Anda tidak wajib mengetik kode secara manual setiap kali. Beberapa versi menerimanya langsung sebagai parameter, meskipun demi keamanan biasanya tidak didukung. Namun Anda tetap bisa mencobanya.

Contoh penggunaan (dalam satu baris):

```
adb pair 192.168.1.42:37123 482915
```

Contoh penggunaan (dengan pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Cara-cara ini berbeda menurut versi. Cara paling andal adalah menjalankan perintah lalu mengetik kode secara manual.

---

5- Hal Yang Perlu Diperhatikan Pada Adb Pair

Port penyambungan dan port koneksi itu berbeda. Port penyambungan biasanya acak dan tinggi, sedangkan port koneksi biasanya 5555. Penyambungan hanya dilakukan sekali; untuk terhubung lagi ke perangkat yang sama, adb connect sudah cukup. Tetapi jika Anda mereset perangkat ke setelan pabrik atau menonaktifkan debugging nirkabel, penyambungan terhapus dan adb pair diperlukan lagi.

Contoh perbandingan:

```
Menyambungkan: adb pair 192.168.1.42:37123   → meminta kode, hanya sekali
Menghubungkan: adb connect 192.168.1.42:5555  → tanpa kode, setiap kali
```

---

6- Perintah Adb Connect

Perintah ini terhubung ke perangkat yang sudah disambungkan. Tulis alamat IP dan port yang ditampilkan di layar utama debugging nirkabel. Port biasanya 5555, tetapi kadang berbeda. Jika koneksi berhasil, muncul pesan Connected to.

Contoh penggunaan:

```
adb connect 192.168.1.42:5555
```

Contoh keluaran:

```
Connected to 192.168.1.42:5555
```

Jika sudah terhubung, ia memberi tahu seperti ini:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Tanpa Port

Di beberapa versi, jika port tidak ditentukan, 5555 dicoba secara bawaan. Tetapi hal ini tidak terjadi sama di semua perangkat, jadi menulis port secara eksplisit adalah yang paling aman.

Contoh penggunaan (port bawaan):

```
adb connect 192.168.1.42
```

Contoh penggunaan (port eksplisit):

```
adb connect 192.168.1.42:5555
```

---

8- Memeriksa Koneksi

Setelah terhubung, perintah adb devices menunjukkan apakah perangkat benar-benar terhubung. Alamat IP perangkat harus muncul di daftar dengan tulisan device di sebelahnya. Jika tertulis offline, koneksi sudah terbentuk tetapi belum stabil; coba lagi sebentar lagi.

Contoh penggunaan:

```
adb devices
```

Contoh keluaran:

```
List of devices attached
192.168.1.42:5555   device
```

Contoh keluaran (tidak terhubung):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- Perintah Adb Disconnect

Memutus koneksi. Koneksi adb berakhir meskipun perangkat tetap berada di jaringan yang sama. Untuk terhubung lagi, jalankan adb connect sekali lagi; tidak perlu menyambungkan ulang.

Contoh penggunaan (satu perangkat):

```
adb disconnect 192.168.1.42:5555
```

Contoh keluaran:

```
Disconnected 192.168.1.42:5555
```

Contoh penggunaan (semua perangkat):

```
adb disconnect
```

Contoh keluaran:

```
Disconnected everything
```

---

10- Perintah Adb Tcpip

Mengalihkan perangkat yang terhubung lewat USB ke mode nirkabel. Perintah ini membuat perangkat mendengarkan pada port yang ditentukan, biasanya 5555. Ini cara yang lebih umum pada versi sebelum Android 11, yang tidak memiliki adb pair.

Contoh penggunaan:

```
adb tcpip 5555
```

Contoh keluaran:

```
Restarting in TCP mode port: 5555
```

Lalu cabut kabel USB dan jalankan perintah ini:

```
adb connect 192.168.1.42:5555
```

---

11- Perintah Adb Usb

Kembali dari mode nirkabel ke mode USB, saat Anda ingin menghubungkan perangkat lagi dengan kabel.

Contoh penggunaan:

```
adb usb
```

Contoh keluaran:

```
Restarting in USB mode
```

---

12- Adb Kill-Server Dan Start-Server

Kadang server adb macet atau menolak koneksi. Saat itu server harus dihentikan lalu dijalankan lagi. Ini mereset semua koneksi dan memberi awal yang bersih.

Contoh penggunaan:

```
adb kill-server
adb start-server
```

Contoh keluaran:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Lalu hubungkan lagi:

```
adb connect 192.168.1.42:5555
```

---

13- Berada Di Jaringan Yang Sama

Untuk adb nirkabel, perangkat dan komputer atau ponsel harus berada di jaringan Wi-Fi yang sama. Di jaringan yang berbeda tidak ada koneksi. Jaringan tamu, pemisahan 5 GHz dan 2,4 GHz, VPN, atau firewall kadang menimbulkan masalah. Solusi sederhananya: hubungkan kedua perangkat ke router yang sama dan matikan VPN.

Contoh pemeriksaan:

```
Perangkat: Setelan → Wi-Fi → Jaringan terhubung (mis. Home_Wifi)
Termux: Setelan → Wi-Fi → Jaringan terhubung (mis. Home_Wifi)
```

---

14- Menemukan Alamat IP Secara Manual

Untuk menemukan alamat IP perangkat, buka Setelan → Wi-Fi → Jaringan terhubung → Detail jaringan. Atau lihat di Termux dengan perintah di bawah. Tetapi alamat IP bisa berubah seiring waktu, jadi yang paling andal adalah memeriksa layar debugging nirkabel setiap kali.

Contoh penggunaan (Termux):

```
ifconfig wlan0
```

Contoh penggunaan (di perangkat):

```
Setelan → Wi-Fi → Jaringan terhubung → Alamat IP
```

---

15- Terhubung Ke Diri Sendiri Dari Termux Di Perangkat Yang Sama

Jika Termux berjalan di perangkat yang sama, adb sudah lokal, tetapi Anda tetap bisa terhubung melalui localhost dengan adb connect. Biasanya dipakai 127.0.0.1:5555. Beberapa perangkat menolaknya; dalam hal itu dipakai IP lokal perangkat itu sendiri.

Contoh penggunaan (localhost):

```
adb connect 127.0.0.1:5555
```

Contoh penggunaan (IP lokalnya):

```
adb connect 192.168.1.42:5555
```

---

16- Menjalankan Perintah Saat Beberapa Perangkat Terhubung

Jika beberapa perangkat terhubung bersamaan, pilih perangkat dengan parameter -s di setiap perintah. Jika tidak, adb gagal dengan error: more than one device.

Contoh penggunaan:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Saat Koneksi Terputus

Jika sinyal Wi-Fi melemah, perangkat masuk mode tidur, atau jaringan berganti, koneksi terputus. Perangkat lalu hilang dari daftar adb devices atau muncul sebagai offline. Solusinya: jalankan adb connect lagi. Penyambungan biasanya tetap berlaku dan adb pair tidak diperlukan.

Contoh pemulihan:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Setelah Perangkat Dimulai Ulang

Saat perangkat dimulai ulang, debugging nirkabel biasanya nonaktif. Di beberapa perangkat tetap aktif, tetapi penyambungan terhapus. Setelah perangkat menyala, buka layar debugging nirkabel, ambil port baru dan kode baru, jalankan adb pair lagi, lalu adb connect.

Contoh alur:

```
Perangkat dimulai ulang → aktifkan debugging nirkabel
Port baru: 42157, kode baru: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Galat Umum Saat Penyambungan

Failed: Wrong password → kode salah ketik.
Failed to connect: Connection refused → port salah atau perangkat tidak mendengarkan.
Failed: Pairing rejected → perangkat menolak penyambungan; perbarui kodenya.
Timeout → ada masalah jaringan; pastikan berada di jaringan yang sama.

Contoh galat dan solusi:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Solusi: kode di perangkat sudah berubah; ambil kode baru dan coba lagi.

---

20- Menonaktifkan Adb Nirkabel

Menonaktifkan debugging nirkabel setelah selesai adalah kebiasaan baik untuk keamanan. Anda bisa menonaktifkannya dari setelan atau beralih ke mode USB dengan adb.

Contoh penggunaan:

```
adb usb
adb disconnect
```

Contoh penggunaan (dari setelan):

```
Setelan → Sistem → Opsi developer → Debugging nirkabel (nonaktif)
```

---

21- Perbedaan Menurut Versi Android

Android 11 dan yang lebih baru: penyambungan dengan adb pair wajib dan port acak dipakai.
Android 10 dan yang lebih lama: beralih ke mode nirkabel dengan adb tcpip 5555; tidak ada penyambungan.

Contoh perbandingan:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Catatan Keamanan

Selama debugging nirkabel aktif, orang berniat jahat di jaringan yang sama bisa mencoba mengakses perangkat. Karena itu, nonaktifkan fitur ini di jaringan Wi-Fi yang tidak Anda percayai. Kode penyambungan ada demi keamanan, tetapi tetap perlu berhati-hati. Nonaktifkan setelah selesai.

Contoh pencegahan:

```
Saat tidak dipakai: Setelan → Opsi developer → Debugging nirkabel (nonaktif)
Di jaringan aman: aktifkan hanya di rumah atau kantor
```

---

23- Seluruh Alur Di Satu Tempat

Skrip di bawah menunjukkan semua langkah, dari nol hingga terhubung. Cukup ganti IP dan port dengan milik perangkat Anda.

Contoh alur:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Hasil yang diharapkan:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Skrip Pintasan Untuk Koneksi Tetap

Jika Anda sering terhubung, Anda bisa menulis skrip dan terhubung setiap kali dengan satu perintah. Misalnya di Termux, Anda bisa menambahkan fungsi ke berkas ~/.bashrc.

Contoh fungsi:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Masukkan IP:port dan kode yang ditampilkan di perangkat:"
    read -p "Alamat: " ADRES
    adb pair "$ADRES"
    read -p "Port koneksi (mis. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Penggunaan:

```
wifi_adb
```

---

25- Catatan Akhir

Adb nirkabel memungkinkan debugging tanpa kabel USB dan sangat praktis saat bekerja di Termux pada ponsel itu sendiri. Dua perintah dasarnya adalah adb pair dan adb connect; yang pertama menyambungkan sekali, yang kedua dipakai di setiap koneksi. Sisanya (adb devices, adb disconnect, adb tcpip, adb kill-server) mendukung kedua perintah itu dan membantu mengatasi masalah. Jangan lupa aturan jaringan yang sama, nonaktifkan saat tidak dipakai, dan mulai ulang server jika ada yang tidak beres. Itu saja.
"#;

const BRAZILIAN: &str = r#"@@ Comandos Do Logcat
1- Adb Logcat

Envia ao vivo para o terminal todos os logs do dispositivo. A partir do momento em que o comando é executado, cada linha de log produzida pelo sistema, pelos aplicativos, pelos serviços e pelo kernel aparece na tela. Para parar, pressione Ctrl + C no teclado. Este comando é a base da depuração; mas, sem filtro, ele pode fazer passar centenas de linhas por segundo e encontrar o que você precisa fica difícil.

Exemplo de uso:

```
adb logcat
```

Exemplo de saída:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example.app
09-21 14:32:01.234  1234  1256 D WindowManager: Adding window
09-21 14:32:01.345  5678  5678 E AndroidRuntime: FATAL EXCEPTION: main
```

---

2- Adb Logcat -D

Mostra os logs já acumulados e o comando termina. Em vez de um fluxo contínuo, tira uma foto daquele momento. Use ao preparar um relatório de bugs, ao salvar o estado depois de uma falha ou quando quiser obter os logs uma única vez a partir de um script.

Exemplo de uso:

```
adb logcat -d
```

Exemplo de uso (salvar em um arquivo):

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
```

Este comando é muito usado em scripts que enviam relatórios automaticamente para um servidor, porque termina sozinho e nunca trava o script.

---

3- Adb Logcat -C

Limpa todos os buffers de log e apaga tudo o que foi acumulado até aquele momento. Se você executá-lo antes de uma nova sessão de depuração, verá apenas os logs produzidos a partir dali e o ruído antigo não vai confundir você. Ao tentar reproduzir um bug, costuma ser o primeiro passo.

Exemplo de uso:

```
adb logcat -c
adb logcat
```

Executando os dois comandos acima em sequência você obtém um fluxo de logs totalmente limpo. Só aparece o que acontece depois da limpeza.

---

4- Adb Logcat -G

Mostra o tamanho dos buffers de log atuais e o quanto estão cheios. Lista, linha por linha, a ocupação de cada buffer, a capacidade total e a quantidade usada. Se os logs passam rápido demais e as entradas antigas são descartadas (isso se chama «chatty»), olhe aqui primeiro.

Exemplo de uso:

```
adb logcat -g
```

Exemplo de saída:

```
main: ring buffer is 256 Kib (250 Kib consumed), max entry is 5120 B
system: ring buffer is 256 Kib (12 Kib consumed), max entry is 5120 B
crash: ring buffer is 256 Kib (0 Kib consumed), max entry is 5120 B
```

---

5- Adb Logcat -G Tamanho

Altera o tamanho do buffer de log. Por exemplo, você pode aumentar um buffer de 256 KiB para 16 MiB. Assim mais logs são guardados e as entradas antigas são descartadas mais tarde. Muito útil em testes longos ou ao depurar um aplicativo muito barulhento. A configuração volta ao padrão quando o dispositivo é reiniciado.

Exemplo de uso:

```
adb logcat -G 16M
```

Exemplo de uso (para diminuir):

```
adb logcat -G 64K
```

---

6- Adb Logcat -F Arquivo

Grava os logs no arquivo indicado em vez de mostrá-los no terminal. O terminal fica limpo e o log é mantido. O caminho do arquivo fica no dispositivo, não em um computador. Em um terminal móvel (Termux) costuma-se gravar em /Sdcard/.

Exemplo de uso:

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt
```

Exemplo de uso (acrescentando ao arquivo):

```
adb logcat -f /Sdcard/Logcat_Kayit.Txt -r 4096 -n 10
```

No exemplo acima um novo arquivo é aberto quando o arquivo chega a 4 MiB, e 10 arquivos são mantidos no total.

---

7- Adb Logcat -B Main

Observa apenas o buffer principal dos aplicativos. Aqui ficam os logs que os aplicativos gravam com chamadas como Log.d, Log.i e Log.e. Ele já é observado por padrão, mas é indicado explicitamente quando você quer só este buffer. É onde mais se olha ao desenvolver um aplicativo.

Exemplo de uso:

```
adb logcat -b main
```

---

8- Adb Logcat -B System

Observa os logs dos serviços do sistema. Aqui ficam os logs de serviços como ActivityManager, WindowManager, PackageManager e PowerManager. Serve para ver eventos como a inicialização de aplicativos, a concessão de permissões e a conexão a serviços. Costuma ser observado junto com main.

Exemplo de uso:

```
adb logcat -b system
```

Exemplo de saída:

```
I ActivityManager: Start proc 1234:com.example/u0a123 for activity
W PackageManager: Permission denied for com.example
```

---

9- Adb Logcat -B Crash

Observa apenas os logs de falhas. Aqui ficam os blocos FATAL EXCEPTION gravados quando um aplicativo é forçado a fechar. Ao depurar uma falha, olhar primeiro para este buffer é o caminho mais rápido, porque o rastreamento da pilha está bem aqui.

Exemplo de uso:

```
adb logcat -b crash
```

Exemplo de saída:

```
E AndroidRuntime: FATAL EXCEPTION: main
E AndroidRuntime: Process: com.example.app, PID: 1234
E AndroidRuntime: Java.Lang.NullPointerException
E AndroidRuntime:   At com.example.app.Main.OnCreate(Main.Java:42)
```

---

10- Adb Logcat -B Radio

Observa os logs de rádio, telefonia, sinal e conexão de dados. Útil ao lidar com o chip SIM, o registro na rede, a conexão 4G/5G, SMS ou chamadas. Normalmente não é necessário no desenvolvimento comum de aplicativos.

Exemplo de uso:

```
adb logcat -b radio
```

---

11- Adb Logcat -B Events

Observa os logs binários dos eventos do sistema. Este buffer não contém texto comum, mas registros curtos codificados deixados pelas chamadas Log.event. Eles aparecem com tags como am_proc_start e am_activity_launch. É usado na análise de desempenho.

Exemplo de uso:

```
adb logcat -b events
```

---

12- Adb Logcat -B Kernel

Observa os logs do kernel. Mostra as mensagens gravadas pelo kernel Linux, que costumam se parecer com a saída do comando dmesg. Aqui estão erros de drivers, problemas de memória e informações sobre temperatura e gerenciamento de energia. Pode exigir root.

Exemplo de uso:

```
adb logcat -b kernel
```

---

13- Adb Logcat -B All

Observa todos os buffers de uma vez. Main, system, crash, radio, events e kernel passam misturados. Útil quando você quer ver tudo, mas a saída fica muito cheia, por isso costuma ser combinado com um filtro.

Exemplo de uso:

```
adb logcat -b all
```

Exemplo de uso (com filtro):

```
adb logcat -b all -s MyApp
```

---

14- Adb Logcat -B Vários Buffers

Observa ao mesmo tempo vários buffers separados por vírgulas, por exemplo para ver juntos os logs do aplicativo e das falhas. Escolhendo só os canais necessários, você reduz a saída.

Exemplo de uso:

```
adb logcat -b main,crash
```

Exemplo de uso (três canais):

```
adb logcat -b main,system,radio
```

---

15- Adb Logcat -V Brief

O formato de saída curto padrão. Cada linha começa apenas com a prioridade e a tag. Bom para uma olhada rápida, mas sem carimbo de hora é difícil ordenar os eventos.

Exemplo de uso:

```
adb logcat -v brief
```

Exemplo de saída:

```
I/ActivityManager( 1234): Start proc com.example
D/MyApp( 5678): Button pressed
E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

16- Adb Logcat -V Time

Acrescenta a data e a hora. Muito mais útil do que o formato curto, porque você vê quando cada evento aconteceu. Abaixo do segundo aparecem os milissegundos.

Exemplo de uso:

```
adb logcat -v time
```

Exemplo de saída:

```
09-21 14:32:01.123 I/ActivityManager( 1234): Start proc com.example
09-21 14:32:01.234 D/MyApp( 5678): Button pressed
09-21 14:32:01.345 E/AndroidRuntime( 5678): FATAL EXCEPTION
```

---

17- Adb Logcat -V Threadtime

O formato de saída mais recomendado. Inclui data, hora, prioridade, tag, PID (identificador do processo) e TID (identificador da thread). Em aplicativos com várias threads é indispensável para saber qual thread fez o quê. É também o formato que o Android Studio usa por padrão.

Exemplo de uso:

```
adb logcat -v threadtime
```

Exemplo de saída:

```
09-21 14:32:01.123  1234  1256 I ActivityManager: Start proc com.example
09-21 14:32:01.234  5678  5678 D MyApp: Button pressed
09-21 14:32:01.345  5678  5690 E AndroidRuntime: FATAL EXCEPTION
```

---

18- Adb Logcat -V Long

Dá uma saída detalhada com todos os campos de metadados e uma linha em branco entre as entradas. Em sessões longas mostra cada log como um bloco, mas ocupa espaço.

Exemplo de uso:

```
adb logcat -v long
```

Exemplo de saída:

```
[ 09-21 14:32:01.123  1234: 1256 I/ActivityManager ]
Start proc com.example

[ 09-21 14:32:01.234  5678: 5678 D/MyApp ]
Button pressed
```

---

19- Adb Logcat -V Color

Colore os logs de acordo com a prioridade. Os erros ficam em vermelho, os avisos em laranja, as informações em verde e a depuração em azul. Em um fluxo longo o olho vai direto ao erro. O terminal precisa suportar cores.

Exemplo de uso:

```
adb logcat -v color
```

Exemplo de uso (junto com time):

```
adb logcat -v time,color
```

---

20- Adb Logcat -V Year

Acrescenta o ano ao formato de hora. Usado em logs longos ou arquivos que cobrem mais de um ano. Normalmente não é necessário no desenvolvimento comum.

Exemplo de uso:

```
adb logcat -v year
```

Exemplo de saída:

```
2026-09-21 14:32:01.123 I/ActivityManager( 1234): Start proc
```

---

21- Adb Logcat -V Uid

Mostra o identificador de usuário (UID) do aplicativo que produziu o log. Útil quando vários aplicativos usam a mesma tag. É um modificador, por isso é combinado com os outros formatos.

Exemplo de uso:

```
adb logcat -v uid
```

Exemplo de saída:

```
I/ActivityManager( 1000): Start proc com.example
D/MyApp(10123): Button pressed
```

---

22- Adb Logcat Filtro De Tag E Prioridade

Mostra os logs de uma determinada tag a partir de uma determinada prioridade. As prioridades são: V (Verbose) a mais baixa, D (Debug), I (Info), W (Warning), E (Error), F (Fatal) e S (Silent) a mais alta. Essa forma de filtrar é o recurso mais poderoso do logcat.

Exemplo de uso (só erros):

```
adb logcat *:E
```

Exemplo de uso (avisos e acima):

```
adb logcat *:W
```

Exemplo de uso (lista de permitidos):

```
adb logcat ActivityManager:I MyApp:D *:S
```

O último exemplo mostra ActivityManager a partir de info e MyApp a partir de debug, e silencia todas as outras tags. Em alguns shells o caractere * é especial, por isso precisa ficar entre aspas:

```
adb logcat "ActivityManager:I MyApp:D *:S"
```

---

23- Adb Logcat -S Tag

Um atalho do filtro silencioso. Mostra só a tag indicada e silencia todo o resto. A forma mais prática de se concentrar na sua própria tag durante o desenvolvimento.

Exemplo de uso:

```
adb logcat -s MyApp
```

Exemplo de uso (várias tags):

```
adb logcat -s MyApp ActivityManager AndroidRuntime
```

No exemplo acima só passam os logs de três tags; todo o resto fica em silêncio.

---

24- Adb Logcat -S Com Prioridade

Serve para indicar juntas a tag e a prioridade, por exemplo para ver só os avisos e acima da tag MyApp.

Exemplo de uso:

```
adb logcat MyApp:W *:S
```

Este comando mostra só os logs warning, error e fatal com a tag MyApp.

---

25- Adb Logcat Filtro --Pid

Mostra só os logs de um determinado identificador de processo (PID). Para descobrir o PID de um aplicativo, usa-se primeiro adb shell pidof e depois filtra-se por esse PID. Perfeito para isolar todos os logs de uma sessão.

Exemplo de uso (obter o PID):

```
adb shell pidof com.example.app
```

Exemplo de saída:

```
5678
```

Exemplo de uso (filtrar):

```
adb logcat --pid=5678
```

Exemplo de uso (em uma linha):

```
adb logcat --pid=$(adb shell pidof com.example.app)
```

---

26- Adb Logcat -T Hora

Mostra os logs a partir do carimbo de hora indicado, ou seja, «traz o que vem depois deste momento». Escreve-se com T maiúsculo; o t minúsculo significa outra coisa.

Exemplo de uso:

```
adb logcat -T "09-21 14:00:00.000"
```

Exemplo de uso (a partir de hoje):

```
adb logcat -T "$(date +'%m-%d %H:%M:%S.000')"
```

---

27- Adb Logcat -T Número De Linhas

Com t minúsculo mostra o número indicado de linhas mais recentes e o comando termina. Funciona como -d, mas dá só as últimas N linhas. Ideal para uma olhada rápida.

Exemplo de uso (últimas 100 linhas):

```
adb logcat -t 100
```

Exemplo de uso (últimas 500 linhas e um filtro):

```
adb logcat -t 500 -s MyApp
```

---

28- Adb Logcat -E Regex

Pesquisa o conteúdo com uma expressão regular (regex) e mostra as linhas que correspondem. Muito poderoso para buscas rápidas de texto; principalmente ao procurar várias palavras de uma vez, é mais rápido do que filtrar.

Exemplo de uso (uma palavra):

```
adb logcat -e "Login"
```

Exemplo de uso (várias palavras):

```
adb logcat -e "Login|Error|Crash"
```

Exemplo de uso (com o nome do pacote):

```
adb logcat -e "com\.example\.app"
```

---

29- Adb Logcat -B Saída Binária

Grava os logs em formato binário. Pessoas não conseguem lê-lo, mas outro logcat pode lê-lo de volta ou outras ferramentas podem processá-lo. É usado em sistemas de análise automática.

Exemplo de uso:

```
adb logcat -B > Ham_Log.Bin
```

Depois esse arquivo é lido assim:

```
adb logcat -B -f Ham_Log.Bin
```

---

30- Adb Logcat -S Estatísticas

Mostra estatísticas dos logs: quantos logs cada tag produziu e quais tags dispararam o aviso «chatty». Se você acha que um aplicativo está inundando o log, confirma isso com este comando.

Exemplo de uso:

```
adb logcat -S
```

Exemplo de saída:

```
Stats:   Total Logs: 12543
         Logs Per Tag:
           ActivityManager: 3421
           MyApp: 8921
           AndroidRuntime: 201
```

---

31- Adb Logcat -R E -N Juntos

Para logs rotativos. -r define o tamanho de cada arquivo e -n quantos arquivos são mantidos. Útil quando um aplicativo vai rodar por muito tempo e você quer registrar continuamente. Os arquivos antigos são apagados e novos são abertos.

Exemplo de uso:

```
adb logcat -f /Sdcard/Log.Txt -r 4096 -n 10
```

No exemplo acima um novo arquivo é aberto sempre que um chega a 4 MiB e 10 arquivos são mantidos no total. Assim o disco nunca enche sem limite.

---

32- Adb Logcat -P Script De Filtro

Indica um script de filtro personalizado para o logcat. É o formato de filtro próprio do Android e costuma ser usado por desenvolvedores do sistema. Desenvolvedores de aplicativos raramente o usam.

Exemplo de uso:

```
adb logcat -P /System/Etc/Logcat_Filter.Script
```

---

33- Adb Logcat -L Carimbos De Hora

Uma opção que, em alguns formatos, acrescenta um carimbo de hora ao início de cada linha. O comportamento muda entre versões do Android. Costuma fazer o mesmo que -v time, mas de outra forma.

Exemplo de uso:

```
adb logcat -L
```

---

34- Adb Logcat -V Printf

Mostra cada linha em formato printf, ou seja, prioridade (PID) tag: mensagem. Serve para compatibilidade com ferramentas antigas, mas raramente é necessário no Android moderno.

Exemplo de uso:

```
adb logcat -v printf
```

---

35- Adb Logcat -V Raw

Mostra só a mensagem bruta, sem metadados. Grava-se apenas o log, sem tag, prioridade nem carimbo de hora. Útil para passar a saída para outro programa.

Exemplo de uso:

```
adb logcat -v raw
```

Exemplo de saída:

```
Start proc com.example
Button pressed
FATAL EXCEPTION
```

---

36- Adb Logcat -V Tag

Grava só a tag no início de cada linha e depois o resto. A prioridade e o PID ficam ocultos. Prático quando você está focado em uma tag.

Exemplo de uso:

```
adb logcat -v tag
```

Exemplo de saída:

```
ActivityManager: Start proc com.example
MyApp: Button pressed
AndroidRuntime: FATAL EXCEPTION
```

---

37- Adb Logcat -V Process

Grava primeiro o PID e depois a tag no início de cada linha. Prático para análise por processo; você vê rapidamente qual processo gravou o quê.

Exemplo de uso:

```
adb logcat -v process
```

Exemplo de saída:

```
1234 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

38- Adb Logcat -V Thread

Grava só o identificador da thread (TID) no início de cada linha e depois o resto. Em análises com várias threads mostra qual thread está ativa. Um formato curto.

Exemplo de uso:

```
adb logcat -v thread
```

Exemplo de saída:

```
1256 I ActivityManager: Start proc com.example
5678 D MyApp: Button pressed
```

---

39- Adb Logcat -D E -T Juntos

Obtém os logs de um intervalo de tempo, por exemplo os da última hora, para salvá-los em um arquivo.

Exemplo de uso:

```
adb logcat -d -T "09-21 13:00:00.000"
```

Exemplo de uso (em um arquivo):

```
adb logcat -d -T "09-21 13:00:00.000" > /Sdcard/Son_Saat.Txt
```

---

40- Adb Logcat -C E -D Em Sequência

Um fluxo que primeiro limpa e depois coleta. Por exemplo, limpar antes de começar um teste e coletar os logs no fim do teste.

Exemplo de uso:

```
adb logcat -c
adb logcat -d > /Sdcard/Test_Sonrasi.Txt
```

---

41- Adb Logcat Salvo Em Arquivo E Levado Para O Computador

Prático para registrar por muito tempo em um terminal móvel. Primeiro grava-se em um arquivo no dispositivo e depois leva-se para o computador com adb pull.

Exemplo de uso (salvar no dispositivo):

```
adb logcat -v threadtime -f /Sdcard/Uzun_Kayit.Txt
```

Exemplo de uso (levar para o computador):

```
adb pull /Sdcard/Uzun_Kayit.Txt ./Uzun_Kayit.Txt
```

---

42- Adb Logcat Com Grep

Usado em um terminal móvel (Termux) para buscar texto rapidamente com grep. Pode ser mais flexível do que o filtro regex, porque tem todas as opções do grep.

Exemplo de uso:

```
adb logcat | grep "Hata"
```

Exemplo de uso (sem diferenciar maiúsculas):

```
adb logcat | grep -I "error"
```

Exemplo de uso (com números de linha):

```
adb logcat | grep -N "Exception"
```

Exemplo de uso (várias palavras):

```
adb logcat | grep -E "Login|Crash|Error"
```

---

43- Adb Logcat Com Tail

Reduz o fluxo ao vivo às suas últimas linhas, quando você só quer ver o final em vez do log inteiro.

Exemplo de uso:

```
adb logcat | tail -100
```

Exemplo de uso (acompanhar continuamente):

```
adb logcat | tail -F
```

---

44- Adb Logcat Com Head

Mostra as primeiras linhas. Bom para uma olhada rápida em um log longo.

Exemplo de uso:

```
adb logcat -d | head -50
```

---

45- Adb Logcat Com Wc

Mostra o número total de linhas, por exemplo para medir quantos logs um teste produziu.

Exemplo de uso:

```
adb logcat -d | wc -l
```

Exemplo de uso (contar só os erros):

```
adb logcat -d *:E | wc -l
```

---

46- Adb Logcat Com Sort

Ordena os logs em ordem alfabética ou numérica. É usado principalmente durante a análise.

Exemplo de uso:

```
adb logcat -d | sort
```

Exemplo de uso (ordem inversa):

```
adb logcat -d | sort -R
```

---

47- Adb Logcat Com Uniq

Remove as linhas repetidas. Se um aplicativo grava sempre o mesmo erro, este comando simplifica a saída.

Exemplo de uso:

```
adb logcat -d | sort | uniq -C | sort -Rn | head -20
```

O exemplo acima conta e mostra as 20 linhas mais repetidas. Muito útil na depuração.

---

48- Adb Logcat Com Awk

Extrai colunas específicas ou processa o texto, por exemplo a coluna PID no formato threadtime.

Exemplo de uso (só a coluna PID):

```
adb logcat -v threadtime | awk '{print $3}'
```

Exemplo de uso (só tag e mensagem):

```
adb logcat -v threadtime | awk '{for(i=6;i<=NF;i++) printf "%s ", $i; print ""}'
```

---

49- Adb Logcat Com Xargs

Executa um comando para cada linha, por exemplo enviar uma notificação para cada linha de erro. É usado principalmente em automação.

Exemplo de uso:

```
adb logcat *:E | xargs -I {} echo "Hata: {}"
```

---

50- Adb Logcat Com Sed

Substitui texto. Ideal para alterar padrões específicos na saída.

Exemplo de uso:

```
adb logcat -v threadtime | sed 's/MyApp/Uygulama/G'
```

O exemplo acima escreve Uygulama no lugar de cada MyApp.

---

51- Adb Logcat Com Tee

Mostra na tela e grava em um arquivo ao mesmo tempo. A observação ao vivo e o registro acontecem juntos.

Exemplo de uso:

```
adb logcat -v threadtime | tee /Sdcard/Canli_Kayit.Txt
```

---

52- Adb Logcat Com Watch

Obtém os logs em intervalos fixos, por exemplo um resumo a cada 5 segundos.

Exemplo de uso:

```
watch -n 5 "adb logcat -d -t 20"
```

---

53- Adb Logcat Com Timeout

Para o logcat depois de um tempo definido. Muito útil em testes automáticos.

Exemplo de uso:

```
timeout 30 adb logcat -v threadtime > /Sdcard/30_Saniye.Txt
```

---

54- Adb Logcat Com Alias

Encurta os comandos que você usa com frequência. No Termux ele é adicionado ao arquivo .bashrc ou .zshrc.

Exemplo de uso (definir um alias):

```
alias lc="adb logcat -v threadtime"
alias lce="adb logcat -v threadtime *:E"
alias lcc="adb logcat -c && adb logcat -v threadtime"
```

Depois basta chamá-lo assim:

```
lce
```

---

55- Adb Logcat Com Vários Dispositivos

Aponta para um dispositivo específico quando há vários conectados. Primeiro liste os dispositivos com adb devices e depois escolha um com -s.

Exemplo de uso:

```
adb devices
```

Exemplo de saída:

```
List of devices attached
Emulator-5554   device
R58M12ABC       device
```

Exemplo de uso (um dispositivo específico):

```
adb -S R58M12ABC logcat -v threadtime
```

---

56- Adb Logcat Com Depuração Por Wi-Fi

Para a depuração por Wi-Fi. Quando o dispositivo está na mesma rede Wi-Fi, a conexão é feita sem cabo. Suportado a partir do Android 11.

Exemplo de uso (ative a depuração por Wi-Fi no dispositivo e depois):

```
adb pair 192.168.1.42:37123
```

Exemplo de uso (conexão):

```
adb connect 192.168.1.42:5555
```

Exemplo de uso (logcat):

```
adb logcat -v threadtime
```

---

57- Adb Logcat Com Bugreport

Cria um relatório detalhado de todo o sistema. Além do logcat são executadas muitas ferramentas como dumpsys, dmesg e top, e tudo vai para um único zip. É a forma de depuração mais completa.

Exemplo de uso:

```
adb bugreport /Sdcard/Bugreport.Zip
```

Exemplo de uso (levar para o computador):

```
adb bugreport ./Bugreport.Zip
```

---

58- Adb Logcat Com Dumpsys

Mostra, junto com o logcat, o estado atual dos serviços do sistema, por exemplo o que o ActivityManager está fazendo naquele momento.

Exemplo de uso:

```
adb shell dumpsys activity activities | head -100
adb logcat -v threadtime
```

---

59- Adb Logcat Com Rastreamentos De ANR

Obtém os rastreamentos de pilha quando um aplicativo não responde (ANR). Os registros de ANR ficam no arquivo /Data/Anr/Traces.Txt.

Exemplo de uso:

```
adb shell cat /Data/Anr/Traces.Txt
```

Exemplo de uso (requer root):

```
adb root
adb shell cat /Data/Anr/Traces.Txt > /Sdcard/Anr.Txt
adb pull /Sdcard/Anr.Txt
```

---

60- Adb Logcat Com Tombstones

Para falhas nativas. Na pasta /Data/Tombstones/ é criado um arquivo para cada falha. É aqui que se procura quando o código C++ ou Rust tem um erro.

Exemplo de uso:

```
adb shell ls /Data/Tombstones/
```

Exemplo de uso (o último tombstone):

```
adb shell cat /Data/Tombstones/Tombstone_00
```

---

61- Adb Logcat Com Systrace

Para o rastreamento de desempenho de todo o sistema. Coleta dados de rastreamento no nível do kernel, não apenas logs. É usado junto com o Perfetto.

Exemplo de uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
```

---

62- Adb Logcat Com Layout Inspector

Analisa o layout de um aplicativo na tela. É usado com o Layout Inspector, mas os logs relacionados também podem ser acompanhados com o logcat.

Exemplo de uso:

```
adb logcat -s LayoutInflater
```

---

63- Adb Logcat Com Memory Profiler

Observa o uso de memória. O Memory Profiler costuma ficar no Android Studio, mas os logs ligados à memória também aparecem no logcat.

Exemplo de uso:

```
adb logcat -s Dalvikvm:V HeapTaskDaemon:V
```

---

64- Adb Logcat Com CPU Profiler

Observa o uso da CPU. Para uma medição simples o comando top também serve.

Exemplo de uso:

```
adb shell top -M 10
adb logcat -v threadtime -s Choreographer
```

---

65- Adb Logcat Com Network Profiler

Observa o tráfego de rede. Bibliotecas como OkHttp e Retrofit gravam no logcat, mas uma análise completa precisa do Network Profiler.

Exemplo de uso:

```
adb logcat -s Okhttp NetworkSecurityConfig
```

---

66- Adb Logcat Com GPU Profiler

Observa o uso da GPU. Para uma medição simples o comando gfxinfo também serve.

Exemplo de uso:

```
adb shell dumpsys gfxinfo com.example.app
adb logcat -s OpenGLRenderer
```

---

67- Adb Logcat Com Battery Historian

Observa o consumo da bateria. Também dá para ver com dumpsys battery.

Exemplo de uso:

```
adb shell dumpsys batterystats > /Sdcard/Battery.Txt
adb logcat -s PowerManagerService
```

---

68- Adb Logcat Para Diagnosticar O Wi-Fi

Serve para depurar problemas de Wi-Fi, como quedas de conexão ou sinal fraco.

Exemplo de uso:

```
adb shell dumpsys wifi
adb logcat -s WifiStateMachine WifiService
```

---

69- Adb Logcat Para Diagnosticar O Bluetooth

Serve para depurar problemas de Bluetooth, como falhas de pareamento ou desconexões.

Exemplo de uso:

```
adb shell dumpsys bluetooth_manager
adb logcat -s BluetoothAdapter BluetoothDevice
```

---

70- Adb Logcat Para Diagnosticar Os Sensores

Serve para depurar problemas de sensores como o acelerômetro, o giroscópio e o magnetômetro.

Exemplo de uso:

```
adb shell dumpsys sensorservice
adb logcat -s SensorService
```

---

71- Adb Logcat Para Diagnosticar A Câmera

Serve para depurar problemas da câmera, como tirar fotos ou gravar vídeo.

Exemplo de uso:

```
adb shell dumpsys media.Camera
adb logcat -s Camera Camera2Client
```

---

72- Adb Logcat Para Diagnosticar O Áudio

Serve para depurar problemas de áudio, como falta de som ou um microfone que não funciona.

Exemplo de uso:

```
adb shell dumpsys audio
adb logcat -s AudioFlinger AudioPolicyManager
```

---

73- Adb Logcat Para Diagnosticar A Localização

Serve para depurar problemas de localização, como falta de sinal de GPS ou uma localização que não se atualiza.

Exemplo de uso:

```
adb shell dumpsys location
adb logcat -s LocationManager GnssLocationProvider
```

---

74- Adb Logcat Para Diagnosticar As Notificações

Serve para depurar problemas de notificações, como notificações que não aparecem ou chegam na hora errada.

Exemplo de uso:

```
adb shell dumpsys notification
adb logcat -s NotificationManager NotificationService
```

---

75- Adb Logcat Para Diagnosticar As Permissões

Serve para depurar problemas de permissões, como uma permissão não concedida ou negada.

Exemplo de uso:

```
adb shell dumpsys package com.example.app | grep -I permission
adb logcat -s PermissionManager PackageManager
```

---

76- Adb Logcat Para Diagnosticar O WebView

Serve para depurar problemas do WebView, como uma página que não carrega ou um erro de JavaScript.

Exemplo de uso:

```
adb logcat -s Chromium WebView
```

Exemplo de uso (mais detalhes):

```
adb logcat -s Cr_Media Cr_Network Cr_Browser
```

---

77- Adb Logcat Para Diagnosticar O OkHttp

Serve para depurar problemas do OkHttp: requisições de rede, respostas e erros.

Exemplo de uso:

```
adb logcat -s Okhttp
```

Exemplo de uso (todos os níveis):

```
adb logcat -s Okhttp:V
```

---

78- Adb Logcat Para Diagnosticar O Retrofit

Serve para depurar problemas do Retrofit. Como ele costuma usar o OkHttp por baixo, olham-se os logs do OkHttp.

Exemplo de uso:

```
adb logcat -s Retrofit Okhttp
```

---

79- Adb Logcat Para Diagnosticar O Glide

Serve para depurar problemas do Glide (biblioteca de carregamento de imagens), como imagens que não carregam ou problemas de memória.

Exemplo de uso:

```
adb logcat -s Glide
```

---

80- Adb Logcat Para Diagnosticar O Picasso

Serve para depurar problemas do Picasso (biblioteca de carregamento de imagens).

Exemplo de uso:

```
adb logcat -s Picasso
```

---

81- Adb Logcat Para Diagnosticar O Room

Serve para depurar problemas do Room (biblioteca de banco de dados): erros de consulta e problemas de esquema.

Exemplo de uso:

```
adb logcat -s Room
```

---

82- Adb Logcat Para Diagnosticar O Firebase

Serve para depurar problemas do Firebase, em módulos como Crashlytics, Analytics e Messaging.

Exemplo de uso:

```
adb logcat -s FirebaseAnalytics FirebaseMessaging FirebaseCrashlytics
```

---

83- Adb Logcat Para Diagnosticar O Crashlytics

Serve para depurar problemas do Firebase Crashlytics.

Exemplo de uso:

```
adb logcat -s FirebaseCrashlytics
```

---

84- Adb Logcat Para Diagnosticar O Analytics

Serve para depurar problemas de análise: eventos não enviados e problemas de envio em lote.

Exemplo de uso:

```
adb logcat -s FirebaseAnalytics GAv4
```

---

85- Adb Logcat Com Depuração ADB Por Wi-Fi

A depuração por Wi-Fi às vezes exige reiniciar o adb. Conecte primeiro por USB e depois passe para o Wi-Fi.

Exemplo de uso:

```
adb tcpip 5555
adb connect 192.168.1.42:5555
adb logcat -v threadtime
```

---

86- Adb Logcat Durante A Reinicialização Do Dispositivo

Reinicia o dispositivo e observa os logs, principalmente para erros que acontecem na inicialização.

Exemplo de uso (dois terminais):

```
Terminal 1: adb logcat -v threadtime
Terminal 2: adb reboot
```

---

87- Adb Logcat Durante A Reinicialização De Um Aplicativo

Reinicia um único aplicativo e observa os logs.

Exemplo de uso:

```
adb shell am force-stop com.example.app
adb logcat -c
adb shell am start -n com.example.app/.MainActivity
adb logcat -v threadtime
```

---

88- Adb Logcat Durante A Instalação De Um Aplicativo

Instala um APK e observa os logs.

Exemplo de uso:

```
adb logcat -c
adb install -R Uygulama.Apk
adb logcat -v threadtime
```

---

89- Adb Logcat Durante A Desinstalação De Um Aplicativo

Desinstala um aplicativo e observa os logs.

Exemplo de uso:

```
adb logcat -c
adb uninstall com.example.app
adb logcat -v threadtime
```

---

90- Adb Logcat Com Captura De Tela

Tira uma captura de tela e salva o log no momento de um erro.

Exemplo de uso:

```
adb logcat -d > /Sdcard/Hata_Logu.Txt
adb shell screencap -P /Sdcard/Ekran_Goruntusu.Png
adb pull /Sdcard/Hata_Logu.Txt
adb pull /Sdcard/Ekran_Goruntusu.Png
```

---

91- Adb Logcat Com Gravação De Tela

Grava a tela e salva o log no momento de um erro. Suportado a partir do Android 4.4.

Exemplo de uso:

```
adb shell screenrecord /Sdcard/Ekran_Kaydi.Mp4 &
adb logcat -v threadtime > /Sdcard/Hata_Logu.Txt
```

---

92- Adb Logcat Com Teste Monkey

Testa um aplicativo simulando interações aleatórias do usuário. Muito eficaz junto com os logs.

Exemplo de uso:

```
adb logcat -c
adb shell monkey -P com.example.app -V 5000
adb logcat -d *:E > /Sdcard/Monkey_Hatalari.Txt
```

---

93- Adb Logcat Com Teste UI Automator

Para testes de automação da interface, observando os logs durante o teste.

Exemplo de uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime > /Sdcard/Ui_Test_Logu.Txt
```

---

94- Adb Logcat Com Teste Espresso

Para testes Espresso, um subconjunto da automação da interface.

Exemplo de uso:

```
adb logcat -c
adb shell am instrument -W -E class Com.Example.EspressoTest Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

95- Adb Logcat Com Teste JUnit

Para testes unitários. Normalmente rodam em um computador, mas também podem rodar no dispositivo.

Exemplo de uso:

```
adb logcat -c
adb shell am instrument -W Com.Example.Test/Androidx.Test.Runner.AndroidJunitrunner
adb logcat -d -V threadtime
```

---

96- Adb Logcat Com StrictMode

Detecta problemas de desempenho de um aplicativo. Avisa quando o disco ou a rede são acessados na thread principal.

Exemplo de uso:

```
adb logcat -s StrictMode
```

---

97- Adb Logcat Com Choreographer

Detecta quadros pulados. É aqui que se procura quando as animações travam.

Exemplo de uso:

```
adb logcat -s Choreographer
```

---

98- Adb Logcat Com Overdraw Da GPU

Detecta problemas de overdraw. É usado na análise do desempenho gráfico.

Exemplo de uso:

```
adb shell dumpsys gfxinfo com.example.app framestats
adb logcat -s GpuOverdraw
```

---

99- Adb Logcat Com A Thread De Renderização

Detecta problemas da thread de renderização. No Android moderno o desenho acontece nessa thread.

Exemplo de uso:

```
adb logcat -s RenderThread
```

---

100- Adb Logcat Com Renderização Da GPU

Detecta problemas de renderização da GPU. É aqui que se procura depois de ativar o perfil de renderização da GPU.

Exemplo de uso:

```
adb shell setprop Debug.Hwui.Profile 1
adb logcat -s OpenGLRenderer
```

---

101- Adb Logcat Com TextView

Detecta problemas das views de texto, como problemas de fonte e erros de medição.

Exemplo de uso:

```
adb logcat -s TextView StaticLayout
```

---

102- Adb Logcat Com RecyclerView

Detecta problemas das views de lista: desempenho da rolagem e reciclagem das views.

Exemplo de uso:

```
adb logcat -s RecyclerView
```

---

103- Adb Logcat Com ViewPager

Detecta problemas nas transições entre páginas.

Exemplo de uso:

```
adb logcat -s ViewPager ViewPager2
```

---

104- Adb Logcat Com Fragments

Detecta problemas do ciclo de vida dos fragments.

Exemplo de uso:

```
adb logcat -s FragmentManager Fragment
```

---

105- Adb Logcat Com Activities

Detecta problemas do ciclo de vida das activities.

Exemplo de uso:

```
adb logcat -s ActivityManager ActivityThread
```

---

106- Adb Logcat Com Serviços

Detecta problemas dos serviços em segundo plano.

Exemplo de uso:

```
adb logcat -s ActivityManager JobScheduler
```

---

107- Adb Logcat Com WorkManager

Detecta problemas do WorkManager, para tarefas em segundo plano.

Exemplo de uso:

```
adb logcat -s WM-WorkerWrapper WM-Processor
```

---

108- Adb Logcat Com AlarmManager

Detecta problemas do AlarmManager, para tarefas agendadas.

Exemplo de uso:

```
adb logcat -s AlarmManager
```

---

109- Adb Logcat Com SharedPreferences

Detecta problemas de SharedPreferences (armazenamento simples de dados).

Exemplo de uso:

```
adb logcat -s SharedPreferences
```

---

110- Adb Logcat Com SQLite

Detecta problemas de SQLite (o banco de dados local).

Exemplo de uso:

```
adb logcat -s SqliteDatabase SqliteConnectionPool
```

---

111- Adb Logcat Com Operações De Arquivos

Detecta problemas de leitura e gravação de arquivos.

Exemplo de uso:

```
adb logcat -s FileUtils FileInputStream FileOutputStream
```

---

112- Adb Logcat Com Operações De Rede

Detecta problemas de conexão de rede, como HTTP, TCP e UDP.

Exemplo de uso:

```
adb logcat -s NetworkController HttpURLConnection
```

---

113- Adb Logcat Com SSL

Detecta problemas de certificados SSL. É aqui que se procura quando as conexões seguras falham.

Exemplo de uso:

```
adb logcat -s SslCertificate SslHandshakeSession
```

---

114- Adb Logcat Com JSON

Detecta problemas de análise de JSON.

Exemplo de uso:

```
adb logcat -s JsonParser JsonReader
```

---

115- Adb Logcat Com XML

Detecta problemas de análise de XML.

Exemplo de uso:

```
adb logcat -s XmlPullParser XmlDocument
```

---

116- Adb Logcat Com Imagens

Detecta problemas de carregamento e processamento de imagens.

Exemplo de uso:

```
adb logcat -s BitmapFactory Bitmap
```

---

117- Adb Logcat Com Mídia

Detecta problemas de reprodução e gravação de mídia.

Exemplo de uso:

```
adb logcat -s MediaPlayer MediaRecorder
```

---

118- Adb Logcat Com ExoPlayer

Detecta problemas do ExoPlayer (biblioteca de player de mídia).

Exemplo de uso:

```
adb logcat -s ExoPlayer
```

---

119- Adb Logcat Com Chromecast

Detecta problemas do Chromecast.

Exemplo de uso:

```
adb logcat -s CastMediaRouteProvider CastDevice
```

---

120- Adb Logcat Com Anúncios

Detecta problemas das bibliotecas de anúncios.

Exemplo de uso:

```
adb logcat -s Ads AdMob
```

---

121- Adb Logcat Com Faturamento

Detecta problemas de faturamento do Google Play.

Exemplo de uso:

```
adb logcat -s BillingClient BillingHelper
```

---

122- Adb Logcat Com Mapas

Detecta problemas das bibliotecas de mapas.

Exemplo de uso:

```
adb logcat -s GoogleMaps Mapbox
```

---

123- Adb Logcat Com Autenticação

Detecta problemas de login, como o Google Sign-In e o Firebase Auth.

Exemplo de uso:

```
adb logcat -s GoogleSignIn FirebaseAuth
```

---

124- Adb Logcat Com Notificações

Detecta problemas de envio e recebimento de notificações.

Exemplo de uso:

```
adb logcat -s NotificationManager FcmReceiver
```

---

125- Adb Logcat Com Deep Links

Detecta problemas de deep links, como abrir um aplicativo a partir de uma URL.

Exemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

126- Adb Logcat Com Intents

Detecta problemas dos intents na passagem entre aplicativos.

Exemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Intent"
```

---

127- Adb Logcat Com Broadcasts

Detecta problemas das mensagens broadcast entre o sistema e os aplicativos.

Exemplo de uso:

```
adb logcat -s BroadcastQueue
```

---

128- Adb Logcat Com Content Providers

Detecta problemas dos content providers, para compartilhar dados entre aplicativos.

Exemplo de uso:

```
adb logcat -s ContentProvider
```

---

129- Adb Logcat Com JobScheduler

Detecta problemas do JobScheduler, para tarefas agendadas em segundo plano.

Exemplo de uso:

```
adb logcat -s JobScheduler
```

---

130- Adb Logcat Com O Modo Soneca

Detecta problemas do modo Soneca (Doze), para o trabalho em segundo plano enquanto o dispositivo está em repouso.

Exemplo de uso:

```
adb logcat -s DeviceIdleController
```

---

131- Adb Logcat Com Otimização Da Bateria

Detecta problemas de otimização da bateria, para as restrições em segundo plano.

Exemplo de uso:

```
adb logcat -s PowerManagerService
```

---

132- Adb Logcat Com Vazamentos De Memória

Detecta vazamentos de memória. É usado junto com ferramentas como o LeakCanary.

Exemplo de uso:

```
adb logcat -s LeakCanary
```

---

133- Adb Logcat Com ANR

Detecta problemas de ANR (o aplicativo não responde).

Exemplo de uso:

```
adb logcat -s ActivityManager | grep -I "Anr"
```

---

134- Adb Logcat Com Falhas

Detecta falhas.

Exemplo de uso:

```
adb logcat -b crash
```

---

135- Adb Logcat Com Falhas Nativas

Detecta falhas nativas (C/C++/Rust).

Exemplo de uso:

```
adb logcat -s DEBUG Libc
```

---

136- Adb Logcat Com Análise De Tombstones

Analisa os registros das falhas nativas.

Exemplo de uso:

```
adb shell ls /Data/Tombstones/
adb shell cat /Data/Tombstones/Tombstone_00
```

---

137- Adb Logcat Com Análise Systrace

Analisa os registros de rastreamento do sistema.

Exemplo de uso:

```
adb shell perfetto -O /Data/Misc/Perfetto-Trace -T 10S -A Raw
adb pull /Data/Misc/Perfetto-Trace
```

---

138- Adb Logcat Com Análise Perfetto

A ferramenta moderna de rastreamento do sistema, mais completa do que o logcat.

Exemplo de uso:

```
adb shell perfetto -C /Data/Misc/Perfetto-Config -O /Data/Misc/Perfetto-Trace -T 10S
```

---

139- Adb Logcat Com Análise Atrace

Para o rastreamento no nível do sistema; a forma mais antiga do Perfetto.

Exemplo de uso:

```
adb shell atrace -Z -T 10 -O /Data/Local/Tmp/Trace.Html gfx view webview
```

---

140- Adb Logcat Com Análise Traceview

Para o profiling no nível dos métodos.

Exemplo de uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
```

---

141- Adb Logcat Com Rastreamento De Métodos

Acompanha um método específico.

Exemplo de uso:

```
adb shell am profile start --sampling 1000 Com.Example.App /Sdcard/Trace.Trace
adb logcat -v threadtime
```

---

142- Adb Logcat Com Heap Dump

Cria um despejo da memória. Muito útil na análise da memória.

Exemplo de uso:

```
adb shell am dumpheap Com.Example.App /Sdcard/Heap.Hprof
adb pull /Sdcard/Heap.Hprof
```

---

143- Adb Logcat Com Log Do GC

Observa os eventos da coleta de lixo.

Exemplo de uso:

```
adb logcat -s Art Dalvikvm | grep -I "Gc"
```

---

144- Adb Logcat Com Depuração JNI

Detecta problemas de JNI (Java Native Interface).

Exemplo de uso:

```
adb logcat -s JNI
```

---

145- Adb Logcat Com Depuração ART

Detecta problemas do ART (Android Runtime).

Exemplo de uso:

```
adb logcat -s Art
```

---

146- Adb Logcat Com Depuração Dalvik

Detecta problemas da máquina virtual Dalvik. É usado nas versões mais antigas do Android.

Exemplo de uso:

```
adb logcat -s Dalvikvm
```

---

147- Adb Logcat E O Problema Chatty

Quando o logcat grava logs demais, ele dá um aviso «chatty» e pula linhas. Isso se resolve com um filtro ou um buffer maior.

Exemplo de uso (aumentar o buffer):

```
adb logcat -G 16M
```

Exemplo de uso (usar um filtro):

```
adb logcat -s MyApp
```

---

148- Adb Logcat Com Saída Colorida

Em fluxos de logs longos, a saída colorida cansa menos os olhos e faz o erro aparecer na hora.

Exemplo de uso:

```
adb logcat -v color
```

Exemplo de uso (com time):

```
adb logcat -v time,color
```

---

149- Adb Logcat Com Um Script Bash

Serve para escrever um script que coleta os logs automaticamente. Em cima dele dá para construir um sistema de relatório de bugs.

Script de exemplo:

```
#!/Data/Data/Com.Termux/Files/Usr/Bin/Bash

set -E

HATA_DIR="/Sdcard/Otomatik_Log"
mkdir -p "$HATA_DIR"

TARIH=$(date +"%Y-%m-%d_%H-%M-%S")
LOG="$HATA_DIR/Logcat_$TARIH.Txt"
CRASH="$HATA_DIR/Crash_$TARIH.Txt"
DUMP="$HATA_DIR/Dumpsys_$TARIH.Txt"

adb logcat -c
adb logcat -v threadtime -f "$LOG" &
LOGCAT_PID=$!

sleep 60

kill $LOGCAT_PID

adb logcat -b crash -d > "$CRASH"
adb shell dumpsys > "$DUMP"

echo "Kayit Tamamlandi: $TARIH"
```

---

150- Adb Logcat Em Resumo

Como esta lista mostra, o logcat não é um único comando, mas um conjunto de comandos. Junto com filtros, formatos, buffers, arquivos e outras ferramentas, ele se torna a arma mais poderosa da depuração. Se você souber qual combinação usar em cada situação, encontrar um bug leva poucos minutos. Lembre-se: a primeira regra de uma sessão de depuração é sempre a mesma; primeiro limpe com adb logcat -c, depois reproduza o bug e depois observe com um logcat filtrado.

@@ Conexão Do Android Debug Bridge
@: Comandos De Conexão ADB Por Wi-Fi
1- Ativar A Depuração Por Wi-Fi

Primeiro é preciso ativar as opções do desenvolvedor no dispositivo. Para isso, toque 7 vezes em Configurações → Sobre o telefone → Número da versão. Depois ative Configurações → Sistema → Opções do desenvolvedor → Depuração por Wi-Fi. Sem esse passo nenhum dos comandos adb pair e adb connect funciona.

Caminho de exemplo:

```
Configurações → Sobre o telefone → Número da versão (tocar 7 vezes)
Configurações → Sistema → Opções do desenvolvedor → Depuração por Wi-Fi (ativada)
```

---

2- Encontrar O Endereço IP E A Porta Do Dispositivo

Com a depuração por Wi-Fi ativada, abra a opção «Parear o dispositivo com um código de pareamento». Ali aparecem o endereço IP do dispositivo e um número de porta aleatório, por exemplo 192.168.1.42:37123. Essa porta serve apenas para o pareamento e depois muda. Na mesma tela há também um código de pareamento de 6 dígitos.

Tela de exemplo:

```
Endereço IP e porta
192.168.1.42:37123

Código de pareamento por Wi-Fi
482915
```

---

3- O Comando Adb Pair

Este comando pareia o dispositivo. É executado em um computador ou no Termux. Digite o endereço IP e a porta de pareamento mostrados no dispositivo. Ao ser executado, o terminal pergunta Enter pairing code: e espera o código de 6 dígitos. Se o código estiver certo, aparece a mensagem Successfully paired.

Exemplo de uso:

```
adb pair 192.168.1.42:37123
```

Exemplo de saída:

```
Enter pairing code: 482915
Successfully paired to 192.168.1.42:37123 [guid=adb-R58M12ABC-Xy7Zq9]
```

Com um código errado ele falha assim:

```
Enter pairing code: 000000
Failed: Wrong password
```

---

4- Outras Formas De Passar O Código Ao Adb Pair

Não é obrigatório digitar o código à mão toda vez. Algumas versões o aceitam direto como parâmetro, embora por segurança isso normalmente não seja suportado. Mesmo assim você pode tentar.

Exemplo de uso (em uma linha):

```
adb pair 192.168.1.42:37123 482915
```

Exemplo de uso (com um pipe):

```
echo "482915" | adb pair 192.168.1.42:37123
```

Esses métodos mudam conforme a versão. A forma mais confiável é executar o comando e digitar o código à mão.

---

5- Cuidados Com O Adb Pair

A porta de pareamento e a porta de conexão são diferentes. A porta de pareamento costuma ser aleatória e alta; a de conexão costuma ser 5555. O pareamento é feito uma única vez; para se conectar de novo ao mesmo dispositivo basta o adb connect. Mas se você restaurar o dispositivo para as configurações de fábrica ou desativar a depuração por Wi-Fi, o pareamento é apagado e será preciso fazer adb pair outra vez.

Comparação de exemplo:

```
Parear:   adb pair 192.168.1.42:37123   → pede um código, só uma vez
Conectar: adb connect 192.168.1.42:5555  → sem código, toda vez
```

---

6- O Comando Adb Connect

Este comando se conecta a um dispositivo pareado. Digite o endereço IP e a porta mostrados na tela principal da depuração por Wi-Fi. A porta costuma ser 5555, mas às vezes é diferente. Se a conexão der certo aparece a mensagem Connected to.

Exemplo de uso:

```
adb connect 192.168.1.42:5555
```

Exemplo de saída:

```
Connected to 192.168.1.42:5555
```

Se já estiver conectado, ele avisa assim:

```
Already connected to 192.168.1.42:5555
```

---

7- Adb Connect Sem Porta

Em algumas versões, se você não indicar a porta, a 5555 é tentada por padrão. Mas isso não acontece do mesmo jeito em todos os dispositivos, por isso escrever a porta explicitamente é o mais seguro.

Exemplo de uso (porta padrão):

```
adb connect 192.168.1.42
```

Exemplo de uso (porta explícita):

```
adb connect 192.168.1.42:5555
```

---

8- Verificar A Conexão

Depois de conectar, o comando adb devices mostra se o dispositivo está mesmo conectado. O endereço IP do dispositivo precisa aparecer na lista com device ao lado. Se estiver escrito offline, a conexão foi estabelecida mas ainda não está estável; tente de novo daqui a pouco.

Exemplo de uso:

```
adb devices
```

Exemplo de saída:

```
List of devices attached
192.168.1.42:5555   device
```

Exemplo de saída (não conectado):

```
List of devices attached
192.168.1.42:5555   offline
```

---

9- O Comando Adb Disconnect

Encerra a conexão. A conexão adb termina mesmo que o dispositivo continue na mesma rede. Para se conectar de novo execute o adb connect outra vez; não é preciso parear.

Exemplo de uso (um dispositivo):

```
adb disconnect 192.168.1.42:5555
```

Exemplo de saída:

```
Disconnected 192.168.1.42:5555
```

Exemplo de uso (todos os dispositivos):

```
adb disconnect
```

Exemplo de saída:

```
Disconnected everything
```

---

10- O Comando Adb Tcpip

Passa para o modo sem fio um dispositivo conectado por USB. Este comando faz o dispositivo escutar na porta indicada, normalmente a 5555. É um método mais comum nas versões anteriores ao Android 11, que não têm adb pair.

Exemplo de uso:

```
adb tcpip 5555
```

Exemplo de saída:

```
Restarting in TCP mode port: 5555
```

Depois desconecte o cabo USB e execute este comando:

```
adb connect 192.168.1.42:5555
```

---

11- O Comando Adb Usb

Volta do modo sem fio para o modo USB, quando você quer conectar o dispositivo de novo com um cabo.

Exemplo de uso:

```
adb usb
```

Exemplo de saída:

```
Restarting in USB mode
```

---

12- Adb Kill-Server E Start-Server

Às vezes o servidor adb trava ou recusa conexões. Nesse caso é preciso pará-lo e iniciá-lo de novo. Isso zera todas as conexões e dá um início limpo.

Exemplo de uso:

```
adb kill-server
adb start-server
```

Exemplo de saída:

```
* Daemon Not Running; Starting Now At Tcp:5037
* Daemon Started Successfully
```

Depois conecte de novo:

```
adb connect 192.168.1.42:5555
```

---

13- Estar Na Mesma Rede

Para o adb sem fio, o dispositivo e o computador ou celular precisam estar na mesma rede Wi-Fi. Em redes diferentes não há conexão. Uma rede de convidados, a divisão entre 5 GHz e 2,4 GHz, uma VPN ou um firewall às vezes causam problemas. A solução simples: conecte os dois aparelhos ao mesmo roteador e desligue as VPNs.

Verificação de exemplo:

```
Dispositivo: Configurações → Wi-Fi → Rede conectada (ex. Home_Wifi)
Termux: Configurações → Wi-Fi → Rede conectada (ex. Home_Wifi)
```

---

14- Encontrar O Endereço IP Manualmente

Para encontrar o endereço IP do dispositivo siga Configurações → Wi-Fi → Rede conectada → Detalhes da rede. Ou veja no Termux com o comando abaixo. Mas o endereço IP pode mudar com o tempo, por isso o mais confiável é conferir sempre a tela da depuração por Wi-Fi.

Exemplo de uso (Termux):

```
ifconfig wlan0
```

Exemplo de uso (no dispositivo):

```
Configurações → Wi-Fi → Rede conectada → Endereço IP
```

---

15- Conectar-Se A Si Mesmo Pelo Termux No Mesmo Dispositivo

Se o Termux roda no mesmo dispositivo, o adb já é local, mas mesmo assim você pode se conectar via localhost com adb connect. Normalmente usa-se 127.0.0.1:5555. Alguns dispositivos recusam; nesse caso usa-se o IP local do próprio dispositivo.

Exemplo de uso (localhost):

```
adb connect 127.0.0.1:5555
```

Exemplo de uso (o seu IP local):

```
adb connect 192.168.1.42:5555
```

---

16- Executar Comandos Com Vários Dispositivos Conectados

Se houver vários dispositivos conectados ao mesmo tempo, escolha o dispositivo com o parâmetro -s em cada comando. Caso contrário o adb falha com error: more than one device.

Exemplo de uso:

```
adb -S 192.168.1.42:5555 logcat
adb -S 192.168.1.43:5555 install Uygulama.Apk
```

---

17- Quando A Conexão Cai

Se o sinal Wi-Fi enfraquecer, o dispositivo entrar em repouso ou a rede mudar, a conexão cai. O dispositivo então some da lista do adb devices ou aparece como offline. A solução: executar o adb connect de novo. O pareamento normalmente continua válido e o adb pair não é necessário.

Recuperação de exemplo:

```
adb disconnect
adb connect 192.168.1.42:5555
adb devices
```

---

18- Depois De Reiniciar O Dispositivo

Quando o dispositivo reinicia, a depuração por Wi-Fi normalmente é desativada. Em alguns dispositivos ela continua ativa, mas o pareamento é apagado. Quando o dispositivo ligar, vá até a tela da depuração por Wi-Fi, pegue a nova porta e o novo código, execute o adb pair de novo e depois o adb connect.

Fluxo de exemplo:

```
Dispositivo reiniciado → ativar a depuração por Wi-Fi
Nova porta: 42157, novo código: 735192
adb pair 192.168.1.42:42157 → 735192
adb connect 192.168.1.42:5555
```

---

19- Erros Comuns No Pareamento

Failed: Wrong password → o código foi digitado errado.
Failed to connect: Connection refused → a porta está errada ou o dispositivo não está escutando.
Failed: Pairing rejected → o dispositivo recusou o pareamento; gere um novo código.
Timeout → há um problema de rede; confira se estão na mesma rede.

Erro de exemplo e solução:

```
adb pair 192.168.1.42:37123
Enter pairing code: 482915
Failed: Wrong password
```

Solução: o código no dispositivo mudou; pegue o novo código e tente de novo.

---

20- Desativar O Adb Sem Fio

Desativar a depuração por Wi-Fi quando terminar é uma boa ideia para a segurança. Você pode desativá-la nas configurações ou passar para o modo USB com o adb.

Exemplo de uso:

```
adb usb
adb disconnect
```

Exemplo de uso (nas configurações):

```
Configurações → Sistema → Opções do desenvolvedor → Depuração por Wi-Fi (desativada)
```

---

21- Diferenças Conforme A Versão Do Android

Android 11 e posteriores: o pareamento com adb pair é obrigatório e usa-se uma porta aleatória.
Android 10 e anteriores: passa-se para o modo sem fio com adb tcpip 5555; não há pareamento.

Comparação de exemplo:

```
Android 11+: adb pair 192.168.1.42:37123 → adb connect 192.168.1.42:5555
Android 10-: adb tcpip 5555 → adb connect 192.168.1.42:5555
```

---

22- Notas De Segurança

Enquanto a depuração por Wi-Fi está ativa, alguém mal-intencionado na mesma rede pode tentar acessar o dispositivo. Por isso mantenha o recurso desativado em redes Wi-Fi em que você não confia. O código de pareamento existe por segurança, mas mesmo assim é bom ter cuidado. Desative quando terminar.

Precaução de exemplo:

```
Quando não estiver usando: Configurações → Opções do desenvolvedor → Depuração por Wi-Fi (desativada)
Em uma rede segura: ative só em casa ou no escritório
```

---

23- Todo O Fluxo Em Um Só Lugar

O script abaixo mostra todos os passos, do zero até a conexão. Basta trocar o IP e a porta pelos do seu dispositivo.

Fluxo de exemplo:

```
adb kill-server
adb start-server
adb pair 192.168.1.42:37123
Enter pairing code: 482915
adb connect 192.168.1.42:5555
adb devices
```

Resultado esperado:

```
List of devices attached
192.168.1.42:5555   device
```

---

24- Um Script De Atalho Para Uma Conexão Duradoura

Se você se conecta com frequência, pode escrever um script e se conectar sempre com um único comando. Por exemplo, no Termux você pode adicionar uma função ao arquivo ~/.bashrc.

Função de exemplo:

```
wifi_adb() {
    adb kill-server
    adb start-server
    echo "Digite o IP:porta e o código mostrados no dispositivo:"
    read -p "Endereço: " ADRES
    adb pair "$ADRES"
    read -p "Porta de conexão (ex. 5555): " PORT
    IP=$(echo "$ADRES" | cut -D: -F1)
    adb connect "$IP:$PORT"
    adb devices
}
```

Uso:

```
wifi_adb
```

---

25- Nota Final

O adb sem fio permite depurar sem cabo USB e é especialmente prático quando se trabalha no Termux no próprio celular. Os dois comandos básicos são adb pair e adb connect; o primeiro pareia uma vez, o segundo é usado a cada conexão. Os demais (adb devices, adb disconnect, adb tcpip, adb kill-server) servem para apoiar esses dois comandos e resolver problemas. Não esqueça a regra da mesma rede, desative quando não estiver usando e reinicie o servidor se algo der errado. É isso.
"#;
