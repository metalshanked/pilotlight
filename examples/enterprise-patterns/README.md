# Enterprise pattern scan fixture

These small source snippets are scanner examples, not runnable applications.
They show a handwritten tool loop, a FastMCP proxy, C# DI and Rust macro declarations.
The configured application ID is the string `0007`. Run from the Pilot Light root:

```powershell
.\bin\pilotlight.exe scan examples/enterprise-patterns --config examples/enterprise-patterns.toml --format csv --output inventory.csv
```

The loop has four evidence rows sharing one asset ID. Inspect the CSV coverage sidecar;
language-specific detection limits still make this scan partial.
