<div class="hero">
	<img src="assets/favicon.png" alt="Aethr logo" class="hero-logo" />
	<h1>Aethr</h1>
	<p class="hero-tagline">Local-first terminal intelligence. Remember every command, fix errors deterministically, and keep sensitive history offline.</p>
	<pre class="hero-command"><code>curl -fsSL https://install.aethr-ai.dev | sh</code></pre>
	<div class="hero-actions">
	<a class="md-button md-button--primary" href="getting-started/installation.md">Get Started</a>
	<a class="md-button" href="https://github.com/aethrAI/aethr" target="_blank" rel="noopener">GitHub Repo</a>
	</div>
	<div class="hero-social">
		<a href="https://x.com/aethr_ai" target="_blank" rel="noopener">@aethr_ai</a>
		<span>•</span>
		<a href="https://discord.gg/XKtbXxG42d" target="_blank" rel="noopener">Discord Community</a>
	</div>
</div>


## Why teams choose Aethr

Aethr keeps engineers fast and confident at the command line while respecting privacy:



## Core principles we refuse to compromise

1. **Local-first by architecture**, not just marketing copy.
2. **Deterministic before probabilistic** — rule engines precede LLM fallbacks.
3. **Transparent ranking** — recency, frequency, and context are always explained.
4. **User agency over data** — no silent collection, ever.


## Source install (power users)

```bash
git clone https://github.com/aethrAI/aethr.git
cd aethr
cargo build --release
./target/release/aethr init
./target/release/aethr import
./target/release/aethr recall "docker build"
```


## Capabilities that ship today

| Area        | Feature                               | Status |
|-------------|----------------------------------------|--------|
| **Search**  | Semantic recall over history           | Stable |
| **Fix**     | Rule + community driven error help     | Stable |
| **Predict** | Intent-based suggestions               | Beta   |
| **Moat**    | Curated shared fixes                   | Stable |
| **Context** | Project tech stack detection           | Stable |
| **LLM Assist** | Advanced reasoning layer            | Planned |


## Coming next on the roadmap

The LLM layer will integrate after deterministic coverage is maximized, unlocking:


We preserve local-first guarantees: remote augmentation is explicit, optional, and transparent.


## Navigate the docs

| Get Started | Dive Deep | Explore | Maintain |
|-------------|-----------|---------|----------|
| [Installation](getting-started/installation.md) | [Architecture](reference/architecture.md) | [Commands](usage/commands.md) | [Troubleshooting](operations/troubleshooting.md) |
| [Quick Start](getting-started/quick-start.md) | [Database](reference/database.md) | [Features](usage/features.md) | [FAQ](operations/faq.md) |


## Design goals we live by


Continue with: [Installation →](getting-started/installation.md)