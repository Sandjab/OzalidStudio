# Ozalid Studio — instructions

Chaîne d'auto-édition, du manuscrit aux packages pour l'imprimeur : intérieur composé,
couverture, planche, dos qui découle de la pagination sans jamais être ressaisi.
Tauri 2 + Rust, front vanilla sans bundler, Typst en sidecar. L'architecture, la
mise en route et le plan du `.ozalid` sont dans `README.md` ; les specs et plans
des chantiers dans `docs/superpowers/`.

`build/` n'est jamais tracké : ressources partagées dans `build/in/{covers,texts,editors}/`,
un répertoire de travail par combinaison texte × couverture (au minimum un `livre.toml`,
dont les chemins partent de `build/`). Le `livre.toml` fait foi pour l'identité du livre
à l'import ; ensuite c'est le `.ozalid` qui la porte.

**Français** dans l'interface, les commentaires et les commits. Termes techniques
anglais conservés tels quels (`fond perdu` reste `fond perdu`, mais `viewport`,
`chunk`, `canvas` ne se traduisent pas).

## Vérifications avant commit

- `cargo fmt --check` et `cargo clippy --all-targets -- -D warnings`, propres.
- `cargo test` (depuis `src-tauri/`) et `node --test tests/*.test.js` (depuis la racine).
- `cargo run --example temoin` si un fichier de `src-tauri/` a changé : le compte
  de pages affiché est le témoin de non-régression, à comparer au précédent sur le
  même manuscrit.
- Tout test neuf doit avoir été **vu échouer** — TDD ou mutation ciblée (spec § 7 de
  chaque chantier) : un test qui n'a jamais été rouge ne protège rien.

## Pièges connus

- Typst est lancé avec `--ignore-system-fonts` : une famille absente des répertoires
  embarqués ne fait pas échouer la composition, elle passe en écriture de repli —
  signalé au compte rendu depuis `typst::compile`, mais en dev `target/debug/fonts`
  ne suit pas `fonts/` tout seul.
- La version de Typst est épinglée : deux versions ne composent pas forcément le même
  nombre de pages, donc pas le même dos. La relever est un changement délibéré, à
  revalider sur un manuscrit réel.
- Le front est embarqué dans le binaire à la compilation : après un changement de
  `src/` seul, `touch src-tauri/src/lib.rs` avant `cargo build`, sinon le binaire
  garde l'ancien front.
- **Les ressources embarquées par `include_str!` / `include_bytes!` — `src-tauri/pods/`
  et `src-tauri/maquettes/` — ne suivent pas mieux que le front**, et leur piège ment
  bien : cargo juge le binaire à jour si leur date précède les artefacts, et fait
  tourner l'**ancien** catalogue contre les nouveaux tests. Le symptôme n'est pas une
  interface périmée mais un écart de valeur — `left: 18.75, right: 18.8` —, c'est-à-dire
  la signature exacte d'une pagination qui aurait bougé. Devant un tel écart, faire
  d'abord `touch src-tauri/pods/*.toml src-tauri/maquettes/*.maquette
  src-tauri/src/lib.rs` et relancer, avant de conclure à une régression du témoin.
