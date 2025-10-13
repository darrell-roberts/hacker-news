# hacker-news-tui

A TUI interface

## Top Stories
```text
┌───────────────────────────────────────────────────Top───────────────────────────────────────────────────┐ ↑
│1  Don't Be a Sucker (1943) [video] by surprisetalk 1 hour ago [10]                                      │ █
│2  NanoChat – The best ChatGPT that $100 can buy by huseyinkeles 6 hours ago [103]                       │ █
│3  First device based on 'optical thermodynamics' can route light without switches by rbanffy 4 days ago │ █
│4  Dutch government takes control of Chinese-owned chipmaker Nexperia by piskov 11 hours ago [122]       │ █
│5  Show HN: SQLite Online – 11 years of solo development, 11K daily users by sqliteonline 9 hours ago [10│ █
│6  Root cause analysis? You're doing it wrong by davedx 2 days ago [28]                                  │ █
│7  Abstraction, not syntax by unripe_syntax 13 hours ago [6]                                             │ █
│8  Modern iOS Security Features – A Deep Dive into SPTM, TXM, and Exclaves by todsacerdoti 3 hours ago   │ █
│9  JIT: So you want to be faster than an interpreter on modern CPUs by pinaraf 1 day ago [1]             │ █
│10 JSON River – Parse JSON incrementally as it streams in by rickcarlino 5 days ago [60]                 │ █
│11 Strudel REPL – a music live coding environment living in the browser by birdculture 3 hours ago [5]   │ █
│12 Scaling request logging with ClickHouse, Kafka, and Vector by mjwhansen 5 days ago [12]               │ █
│13 Android's sideloading limits are its most anti-consumer move by josephcsible 6 hours ago [283]        │ █
│14 CRDT and SQLite: Local-First Value Synchronization by marcobambini 4 days ago [9]                     │ ║
│15 Software update bricks some Jeep 4xe hybrids over the weekend by gloxkiqcza 7 hours ago [180]         │ ║
│16 Optery (YC W22) – Hiring Tech Lead with Node.js Experience (U.S. & Latin America) by beyondd 4 hours a│ ║
│17 Systems as Mirrors by i8s 1 day ago                                                                   │ ║
│18 Reverse Engineering a 1979 Camera's Spec by manoloesparta 3 hours ago [2]                             │ ║
│19 Spotlight on pdfly, the Swiss Army knife for PDF files by Lucas-C 13 hours ago [88]                   │ ║
│20 Roger Dean – His legendary artwork in gaming history (Psygnosis) by thelok 7 hours ago [13]           │ ║
│21 American solar farms by marklit 11 hours ago [207]                                                    │ ║
│22 Smartphones and being present by articsputnik 7 hours ago [108]                                       │ ║
│23 Matrices can be your friends (2002) by todsacerdoti 11 hours ago [83]                                 │ ║
│24 More random home lab things I've recently learned by otter-in-a-suit 7 days ago [89]                  │ ║
│25 AWS Service Availability Updates by dabinat 2 hours ago [8]                                           │ ║
│26 Environment variables are a legacy mess: Let's dive deep into them by signa11 5 hours ago [137]       │ ║
│27 The Sveriges Riksbank Prize in Economic Sciences in Memory of Alfred Nobel 2025 by k2enemy 10 hours ag│ ║
│28 MPTCP for Linux by SweetSoftPillow 12 hours ago [18]                                                  │ ║
│29 Ancient Patagonian hunter-gatherers took care of their injured and disabled by pseudolus 6 days ago [6│ ║
│30 KTX – npx for Kotlin and JVM to install jars or Kotlin scripts by TheWiggles 5 days ago               │ ║
│31 Some graphene firms have reaped its potential but others are struggling by robaato 13 hours ago [32]  │ ║
│32 German state replaces Microsoft Exchange and Outlook with open-source email by CrankyBear 2 hours ago │ ║
│33 Programming in Assembly Is Brutal, Beautiful, and Maybe Even a Path to Better AI by fcpguru 3 hours ag│ ║
│34 LLMs are getting better at character-level text manipulation by curioussquirrel 2 hours ago           │ ║
│35 Control your Canon Camera wirelessly by nklswbr 6 days ago [17]                                       │ ║
│36 Putting a dumb weather station on the internet by todsacerdoti 6 days ago [37]                        │ ║
│37 Clockss: Digital preservation services run by academic publishers and libraries by robtherobber 5 days│ ║
└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ↓
https://github.com/karpathy/nanochat
Index (13/10/25 17:49) (13 sec, 15 ms)                                                   Total comments: 4965
```
## View comments

```
NanoChat – The best ChatGPT that $100 can buy
https://x.com/karpathy/status/1977755427569111362

┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  ▲
│Interesting exchange on the use of AI coding tools:                                                     │  █
│                                                                                                        │  █
│    curious how much did you write the code by hand of it?                                              │  █
│                                                                                                        │  █
│    Karpathy: Good question, it's basically entirely hand-written (with tab autocomplete). I tried to   │  █
│use claude/codex agents a few times but they just didn't work well enough at all and net unhelpful,     │  █
│possibly the repo is too far off the data distribution.                                                 │  █
│https://x.com/karpathy/status/1977758204139331904                                                       │  █
│                                                                                                        │  █
└────────────────────────────────────────────────────────────────────────────by tehnub 27 minutes ago [1]┘  █
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  █
│> nanochat is also inspired by modded-nanoGPT                                                           │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│Nice synergy here, the lineage is: Karpathy's nano-GPT -> Keller Jordan's modded-nanoGPT (a speedrun of │  ║
│training nanoGPT) -> NanoChat                                                                           │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│modded-nanoGPT [1] is a great project, well worth checking out, it's all about massively speeding up the│  ║
│training of a small GPT model.                                                                          │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│Notably it uses the author's Muon optimizer [2], rather than AdamW, (for the linear layers).            │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│[1] https://github.com/KellerJordan/modded-nanogpt                                                      │  ║
│                                                                                                        │  ║
│[2] https://kellerjordan.github.io/posts/muon/                                                          │  ║
│                                                                                                        │  ║
└────────────────────────────────────────────────────────────────────────by montebicyclelo 1 hour ago [3]┘  ║
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  ║
│Nice! His Shakespeare generator was one of the first projects I tried after ollama. The goal was to     │  ║
│understand what LLMs were about.                                                                        │  ║
│                                                                                                        │  ▼
                                                   1 2 3
https://github.com/karpathy/nanochat
Index (13/10/25 17:49) (13 sec, 15 ms)                                                   Total comments: 4965
```

## Search Index
```
┌Search─────────────────────────────────────────────────────────────────────────────────────────────────────┐
│Rust                                                                                                       │
└───────────────────────────────────────────────────────────────────────────────────────────────────────────┘
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  ▲
│This looks very exciting! I'm following it and I'll give it a go. Not that I'm unsatisfied with Claude  │  █
│Code for my amateur level, but it's clear incentives are not exactly aligned when using a tool from the │  █
│token provider xD                                                                                       │  █
│                                                                                                        │  █
│I love that you've made it open source and that it's in Rust, thanks a lot for the work!                │  █
│                                                                                                        │  █
└───────────────────────────────────────────────────────────────────────────by OldOneEye 14 hours ago [1]┘  █
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  █
│Thank you for your kind words. This is my own research into how coding agent works in practice, I love  │  █
│to explore the underlying technologies of how Claude Code, and Codex and coding agent works in general. │  █
│                                                                                                        │  ║
│I choose Rust since I have some familiarity and experience with it, VT Code is of course, AI-assisted, I│  ║
│mainly use Codex to help me build it. Thank you again for checking it out, have a great day! : )        │  ║
│                                                                                                        │  ║
└──────────────────────────────────────────────────────────────────────────────────by vinhnx 14 hours ago┘  ║
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐  ║
│As a Turkish speaker who was using a Turkish-locale setup in my teenage years these kinds of bugs       │  ║
│frustrated me infinitely. Half of the Java or Python apps I installed never run. My PHP webservers      │  ║
│always had problems with random software. Ultimately, I had to change my system's language to English.  │  ║
│However, US has godawful standards for everything: dates, measurement units, paper sizes.               │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│When I shared computers with my parents I had to switch languages back-and-forth all the time. This     │  ║
│helped me learn English rather quickly but, I find it a huge accessibility and software design issue.   │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│If your program depends on letter cases, that is a badly designed program, period. If a language ships  │  ║
│toUpper or a toLower function without a mandatory language field, it is badly designed too. The only    │  ║
│slightly-better option is making toUpper and toLower ASCII-only and throwing error for any other        │  ║
│character set.                                                                                          │  ║
│                                                                                                        │  ║
│                                                                                                        │  ║
│While half of the language design of C is questionable and outright dangerous, making its functions     │  ║
│locale-sensitive by all popular OSes was an avoidable mistake. Yet everybody did that. Just the         │  ▼
                                                1 2 3 4 5 6
Found 51
Index (13/10/25 17:49) (13 sec, 15 ms)                                                   Total comments: 4965
```
