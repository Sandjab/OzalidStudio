# Version portable Windows — plan d'implémentation

> **Pour un agent exécutant :** SOUS-SKILL REQUIS — utiliser
> `superpowers:subagent-driven-development` (recommandé) ou
> `superpowers:executing-plans` pour dérouler ce plan tâche par tâche. Les étapes
> sont des cases à cocher (`- [ ]`).

**But :** livrer, à côté de l'installeur NSIS, une archive `.zip` que l'on déplie où
l'on veut et qui garde ses réglages dans un `donnees/` voisin de l'exécutable plutôt
que sur la machine hôte.

**Architecture :** un module `emplacement.rs` répond seul à la question « où
l'application écrit-elle ce qui n'appartient pas à un livre ? ». Il décide à partir de
la présence d'un marqueur posé à côté de l'exécutable, une fois, au `setup` ; les trois
consommateurs actuels de `app_config_dir()` lisent ce résultat en état Tauri managé au
lieu de reposer la question. L'archive est assemblée par `outils/portable.sh`, et la CI
interroge le binaire livré par un drapeau de diagnostic plutôt que de la croire sur
parole.

**Pile :** Rust + Tauri 2, front vanilla sans bundler, tests `cargo test --lib` et
`node --test tests/*.test.js`, CI GitHub Actions sur `windows-latest`.

**Spec :** `docs/superpowers/specs/2026-08-31-windows-portable-design.md`

## Contraintes globales

- **Français** dans l'interface, les commentaires et les commits ; termes techniques
  anglais conservés tels quels.
- **Avant chaque commit** : `cargo fmt --check` et `cargo clippy --all-targets -- -D
  warnings` propres, `cargo test` depuis `src-tauri/`, `node --test tests/*.test.js`
  depuis la racine.
- **Le glob des tests front se donne sans guillemets.** `node --test "tests/*.test.js"`
  — la forme du CLAUDE.md et de la CI — rend **0 test** en silence sur le Node de ce
  poste (v26). Vérifié le 31/08/2026 : 331 tests sans guillemets, 0 avec. Un compte de
  tests à zéro n'est jamais un succès : le lire comme tel est le seul moyen de commiter
  un front cassé en croyant l'avoir testé.
- **Tout test neuf doit avoir été vu échouer.** Les tâches ci-dessous nomment
  l'échec attendu à chaque fois ; ne pas passer à l'implémentation sans l'avoir lu.
- **Le marqueur** : nom de l'exécutable privé de son extension, suffixé `.portable`.
  Pour `ozalid-studio.exe` → `ozalid-studio.portable`. Fichier vide, contenu jamais lu.
- **Le sous-dossier de données** : `donnees`, à côté de l'exécutable.
- **Le drapeau de diagnostic** : `--emplacement <fichier>`. Il écrit dans un fichier et
  jamais sur la sortie standard — `main.rs:2` pose `windows_subsystem = "windows"` en
  release, l'exécutable n'a aucune console rattachée.
- **Aucune dépendance nouvelle** dans `src-tauri/Cargo.toml`.
- **Ne pas toucher au job `verifier`** de `.github/workflows/windows.yml`.
- Après la tâche 3, `app_config_dir` ne doit plus apparaître **qu'une seule fois** dans
  tout `src-tauri/src/`.

---

### Tâche 1 : `emplacement.rs`, la décision prise une fois

**Fichiers :**
- Créer : `src-tauri/src/emplacement.rs`
- Modifier : `src-tauri/src/lib.rs` (liste des `pub mod`, ordre alphabétique — entre
  `ean` et `envoi`)

**Interfaces :**
- Consomme : rien.
- Produit : `emplacement::Mode` (`Installe` | `Portable` | `PortableLectureSeule`,
  `Copy`, `serde::Serialize` en kebab-case), `emplacement::Emplacement { pub racine:
  Option<PathBuf>, pub mode: Mode }`, `emplacement::resoudre(systeme: Option<PathBuf>)
  -> Emplacement`, et `emplacement::depuis_executable(exe: &Path, systeme:
  Option<PathBuf>) -> Emplacement` (visibilité `pub(crate)`, le point d'entrée des
  tests).

- [ ] **Étape 1 : écrire les quatre tests, qui ne compilent pas encore**

Créer `src-tauri/src/emplacement.rs` avec **seulement** ce bloc de tests :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Un dossier jetable qui joue le voisinage de l'exécutable. Rend le chemin de
    /// l'exécutable fictif : `depuis_executable` ne le lit jamais, il n'a pas besoin
    /// d'exister.
    fn atelier() -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let exe = d.path().join("ozalid-studio.exe");
        (d, exe)
    }

    /// **La régression la plus coûteuse du chantier.** Sans marqueur, l'application
    /// doit écrire là où elle a toujours écrit. Un `Portable` rendu ici enverrait les
    /// réglages d'une installation existante dans son dossier d'installation.
    #[test]
    fn sans_marqueur_la_racine_reste_celle_du_systeme() {
        let (_d, exe) = atelier();
        let systeme = PathBuf::from("/config/systeme");
        let e = depuis_executable(&exe, Some(systeme.clone()));
        assert_eq!(e.mode, Mode::Installe);
        assert_eq!(e.racine, Some(systeme));
    }

    /// Le marqueur posé, tout ce qui n'appartient pas à un livre descend dans un
    /// `donnees/` voisin — créé au passage, l'archive ne le livre pas.
    #[test]
    fn avec_marqueur_la_racine_est_le_dossier_de_donnees() {
        let (d, exe) = atelier();
        std::fs::write(d.path().join("ozalid-studio.portable"), b"").unwrap();
        let e = depuis_executable(&exe, Some(PathBuf::from("/config/systeme")));
        assert_eq!(e.mode, Mode::Portable);
        assert_eq!(e.racine, Some(d.path().join("donnees")));
        assert!(d.path().join("donnees").is_dir(), "le dossier doit être créé");
    }

    /// Le marqueur est nommé d'après l'exécutable, et pas seulement suffixé : sans ce
    /// test, n'importe quel `*.portable` traînant dans le dossier basculerait
    /// l'application — un fichier laissé par un autre outil, ou une copie renommée.
    #[test]
    fn un_marqueur_qui_ne_porte_pas_le_nom_de_l_executable_ne_compte_pas() {
        let (d, exe) = atelier();
        std::fs::write(d.path().join("autre.portable"), b"").unwrap();
        let e = depuis_executable(&exe, Some(PathBuf::from("/config/systeme")));
        assert_eq!(e.mode, Mode::Installe);
    }

    /// Support en lecture seule : on le dit, et on reste portable. La racine est
    /// **servie quand même**, sans quoi lire ce qui est déjà là deviendrait impossible
    /// — et c'est elle, non le mode, qui distingue cette décision du repli silencieux
    /// sur le répertoire du système, écarté au cadrage.
    ///
    /// L'empêchement est obtenu en posant un *fichier* nommé `donnees` : `create_dir_all`
    /// y échoue sur toutes les plateformes, là où un jeu de permissions POSIX ne dirait
    /// rien sous Windows — où ce mode a précisément lieu d'exister.
    #[test]
    fn un_dossier_de_donnees_impossible_laisse_en_lecture_seule() {
        let (d, exe) = atelier();
        std::fs::write(d.path().join("ozalid-studio.portable"), b"").unwrap();
        std::fs::write(d.path().join("donnees"), b"pas un dossier").unwrap();
        let e = depuis_executable(&exe, Some(PathBuf::from("/config/systeme")));
        assert_eq!(e.mode, Mode::PortableLectureSeule);
        assert_eq!(
            e.racine,
            Some(d.path().join("donnees")),
            "la racine reste servie : lire ce qui est déjà là doit rester possible"
        );
    }
}
```

Déclarer le module dans `src-tauri/src/lib.rs`, à sa place alphabétique :

```rust
pub mod ean;
pub mod ebook;
pub mod emplacement;
pub mod empreinte;
```

- [ ] **Étape 2 : voir l'échec**

```bash
cd src-tauri && cargo test --lib emplacement
```

Attendu : **échec de compilation**, `cannot find function depuis_executable in this
scope` (et de même pour `Mode`, `PathBuf`).

- [ ] **Étape 3 : écrire le module**

Insérer au-dessus du bloc `#[cfg(test)]` :

```rust
//! Où l'application écrit ce qui n'appartient pas à un livre.
//!
//! Trois choses vivent hors des projets — `preferences.toml`, le dossier `maquettes/`
//! et les surcharges de catalogue — et elles descendent toutes du même `&Path`. Ce
//! module est le seul endroit qui décide de ce chemin, et il décide **une fois** : le
//! `setup` résout, les trois consommateurs lisent. C'est ce qui empêche un quatrième
//! appelant d'ouvrir un chemin parallèle sans qu'on le voie.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// L'extension du marqueur, accolée au nom de l'exécutable privé de la sienne.
const MARQUEUR: &str = "portable";

/// Le sous-dossier où descend tout ce que le mode portable écrit. Un sous-dossier et
/// non le dossier de l'exécutable : ce qui vient de l'archive et ce que l'usage a
/// produit restent distinguables, et l'un se sauvegarde sans l'autre.
const DONNEES: &str = "donnees";

/// Le fichier témoin de l'essai d'écriture. Écrit puis effacé, il ne survit à rien.
const TEMOIN: &str = ".acces";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Pas de marqueur : le répertoire de configuration du système, comme toujours.
    Installe,
    /// Marqueur posé, et le dossier de données accepte l'écriture.
    Portable,
    /// Marqueur posé, mais rien ne pourra être enregistré. L'interface le dit.
    PortableLectureSeule,
}

/// Où écrire, et sous quel régime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emplacement {
    /// `None` quand même le système n'en propose pas : l'application démarre alors sur
    /// les défauts, ce que `preferences::charger` et `catalogue::charge` savent déjà
    /// faire.
    pub racine: Option<PathBuf>,
    pub mode: Mode,
}

/// Résout l'emplacement pour l'exécutable en cours.
///
/// `systeme` est ce que Tauri propose (`app.path().app_config_dir().ok()`), passé en
/// argument plutôt que lu ici : c'est ce qui rend tout ce module testable sans Tauri.
pub fn resoudre(systeme: Option<PathBuf>) -> Emplacement {
    match std::env::current_exe() {
        Ok(exe) => depuis_executable(&exe, systeme),
        // Un environnement sans exécutable au sens usuel ne doit pas empêcher le
        // démarrage : il n'est simplement pas portable.
        Err(_) => Emplacement {
            racine: systeme,
            mode: Mode::Installe,
        },
    }
}

pub(crate) fn depuis_executable(exe: &Path, systeme: Option<PathBuf>) -> Emplacement {
    let installe = Emplacement {
        racine: systeme,
        mode: Mode::Installe,
    };
    let (Some(dossier), Some(nom)) = (exe.parent(), exe.file_stem()) else {
        return installe;
    };
    // `format!` et non `with_extension` : un exécutable dont le nom porte un point
    // verrait `with_extension` lui manger sa fin.
    let marqueur = dossier.join(format!("{}.{MARQUEUR}", nom.to_string_lossy()));
    if !marqueur.is_file() {
        return installe;
    }
    let racine = dossier.join(DONNEES);
    let mode = if inscriptible(&racine) {
        Mode::Portable
    } else {
        Mode::PortableLectureSeule
    };
    Emplacement {
        racine: Some(racine),
        mode,
    }
}

/// Écrit vraiment, plutôt que d'interroger des permissions : sous Windows, un attribut
/// de fichier ne dit pas ce qu'un partage réseau ou une stratégie de groupe autorisera.
/// La seule réponse fiable est la tentative.
fn inscriptible(racine: &Path) -> bool {
    if std::fs::create_dir_all(racine).is_err() {
        return false;
    }
    let temoin = racine.join(TEMOIN);
    if std::fs::write(&temoin, b"").is_err() {
        return false;
    }
    let _ = std::fs::remove_file(&temoin);
    true
}
```

- [ ] **Étape 4 : voir passer**

```bash
cd src-tauri && cargo test --lib emplacement
```

Attendu : **4 tests passés**.

- [ ] **Étape 5 : prouver que les tests mordent (mutation ciblée)**

Trois mutations, une à la fois, à annuler après chaque lecture :

| Mutation | Test qui doit tomber |
|---|---|
| remplacer `if !marqueur.is_file()` par `if false` | `sans_marqueur_la_racine_reste_celle_du_systeme` |
| remplacer le `format!` par `dossier.join(format!("x.{MARQUEUR}"))` | `un_marqueur_qui_ne_porte_pas_le_nom_de_l_executable_ne_compte_pas` |
| dans la branche `PortableLectureSeule`, rendre `racine: None` | `un_dossier_de_donnees_impossible_laisse_en_lecture_seule` |

Lancer `cargo test --lib emplacement` après chaque mutation, **lire le rouge**, puis
rétablir. Une mutation qui laisse tout vert est un test à réécrire, pas une mutation à
abandonner.

- [ ] **Étape 6 : contrôles et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --lib
cd .. && git add src-tauri/src/emplacement.rs src-tauri/src/lib.rs
git commit -m "Un marqueur voisin de l'exécutable décide où vont les réglages"
```

---

### Tâche 2 : le drapeau `--emplacement`

**Fichiers :**
- Modifier : `src-tauri/src/emplacement.rs` (deux fonctions et leurs tests)
- Modifier : `src-tauri/src/lib.rs` (première ligne de `run()`)

**Interfaces :**
- Consomme : `emplacement::resoudre`, `emplacement::Emplacement`, `emplacement::Mode`
  (tâche 1).
- Produit : `emplacement::sortie_demandee(args: impl IntoIterator<Item = String>) ->
  Option<PathBuf>` et `emplacement::rapport(e: &Emplacement) -> String`. La CI (tâche 6)
  s'appuie sur le format exact du rapport.

- [ ] **Étape 1 : écrire les tests, dans le `mod tests` existant**

```rust
    /// L'argument est reconnu, et le chemin qui le suit est celui du rapport.
    #[test]
    fn le_drapeau_donne_le_fichier_de_sortie() {
        let args = ["--emplacement", "rapport.txt"].map(String::from);
        assert_eq!(
            sortie_demandee(args),
            Some(PathBuf::from("rapport.txt"))
        );
    }

    /// Un lancement ordinaire — et un lancement où Windows ajoute ses propres
    /// arguments — n'en demande aucun. Le drapeau ne fait pas de cet exécutable une
    /// commande : ce qu'il ne reconnaît pas, il l'ignore.
    #[test]
    fn sans_le_drapeau_aucun_rapport_n_est_demande() {
        assert_eq!(sortie_demandee(Vec::<String>::new()), None);
        assert_eq!(
            sortie_demandee(["/un/chemin.ozalid"].map(String::from)),
            None
        );
        // Le drapeau sans son chemin ne rend rien plutôt que d'écrire n'importe où.
        assert_eq!(sortie_demandee(["--emplacement"].map(String::from)), None);
    }

    /// Le format que la CI relit. Il nomme le mode, et la racine quand elle existe.
    #[test]
    fn le_rapport_nomme_le_mode_et_la_racine() {
        let e = Emplacement {
            racine: Some(PathBuf::from("/cle/Ozalid/donnees")),
            mode: Mode::Portable,
        };
        let r = rapport(&e);
        assert!(r.contains("mode = portable\n"), "rapport : {r}");
        assert!(r.contains("/cle/Ozalid/donnees"), "rapport : {r}");
    }

    /// En mode installé, le rapport ne ment pas sur un chemin qu'il n'a pas : le
    /// drapeau s'exécute avant que Tauri n'ait résolu quoi que ce soit. Il dit le mode,
    /// et renvoie au système.
    #[test]
    fn le_rapport_installe_ne_fabrique_pas_de_chemin() {
        let e = Emplacement {
            racine: None,
            mode: Mode::Installe,
        };
        let r = rapport(&e);
        assert!(r.contains("mode = installe\n"), "rapport : {r}");
        assert!(r.contains("système"), "rapport : {r}");
    }
```

- [ ] **Étape 2 : voir l'échec**

```bash
cd src-tauri && cargo test --lib emplacement
```

Attendu : **échec de compilation**, `cannot find function sortie_demandee` et `cannot
find function rapport`.

- [ ] **Étape 3 : écrire les deux fonctions**

À ajouter dans `emplacement.rs`, avant le `mod tests` :

```rust
/// Le fichier de rapport demandé par `--emplacement <fichier>`, s'il l'est.
///
/// Reçoit les arguments plutôt que de lire `std::env::args` : sans quoi cette fonction
/// ne se testerait qu'en modifiant l'environnement du processus de test.
pub fn sortie_demandee(args: impl IntoIterator<Item = String>) -> Option<PathBuf> {
    let mut args = args.into_iter();
    while let Some(a) = args.next() {
        if a == "--emplacement" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

/// Ce que `--emplacement` écrit : le mode, puis la racine.
///
/// Dans un fichier et non sur la sortie standard — `main.rs` pose
/// `windows_subsystem = "windows"` en release, l'exécutable n'a aucune console où
/// écrire. C'est aussi la façon dont `examples/temoin.rs` prend sa sortie.
pub fn rapport(e: &Emplacement) -> String {
    let mode = match e.mode {
        Mode::Installe => "installe",
        Mode::Portable => "portable",
        Mode::PortableLectureSeule => "portable-lecture-seule",
    };
    let racine = match &e.racine {
        Some(r) => r.display().to_string(),
        None => "répertoire de configuration du système".to_owned(),
    };
    format!("mode = {mode}\nracine = {racine}\n")
}
```

- [ ] **Étape 4 : voir passer**

```bash
cd src-tauri && cargo test --lib emplacement
```

Attendu : **8 tests passés**.

- [ ] **Étape 5 : brancher le drapeau dans `run()`**

Dans `src-tauri/src/lib.rs`, en **toute première instruction** de `pub fn run()`, avant
`tauri::Builder::default()` :

```rust
pub fn run() {
    // Avant Tauri, délibérément : ce drapeau répond sans ouvrir de fenêtre, et c'est
    // par lui que la CI interroge l'archive réellement livrée. En mode installé il ne
    // peut pas nommer le répertoire du système — le résolveur de Tauri n'existe pas
    // encore — et le rapport le dit plutôt que de l'inventer.
    if let Some(sortie) = emplacement::sortie_demandee(std::env::args().skip(1)) {
        let rapport = emplacement::rapport(&emplacement::resoudre(None));
        if let Err(e) = std::fs::write(&sortie, rapport) {
            // Sans console sous Windows, personne ne lira ce message ; le code de
            // sortie, lui, se lit — et c'est sur lui que la CI s'arrête.
            eprintln!("rapport d'emplacement ({}) : {e}", sortie.display());
            std::process::exit(1);
        }
        return;
    }

    tauri::Builder::default()
```

- [ ] **Étape 6 : prouver le drapeau sur le binaire local**

```bash
cd src-tauri && cargo build
S=$(mktemp -d)
./target/debug/ozalid-studio --emplacement "$S/rapport.txt" && cat "$S/rapport.txt"
```

Attendu : `mode = installe` et la ligne de renvoi au système. Aucune fenêtre ne s'ouvre.

Puis le cas portable, sur le binaire de développement :

```bash
cd src-tauri && touch target/debug/ozalid-studio.portable
./target/debug/ozalid-studio --emplacement "$S/portable.txt" && cat "$S/portable.txt"
```

Attendu : `mode = portable` et une racine se terminant par `target/debug/donnees`.
Puis nettoyer : `rm target/debug/ozalid-studio.portable && rm -rf target/debug/donnees`.

- [ ] **Étape 7 : contrôles et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --lib
cd .. && git add src-tauri/src/emplacement.rs src-tauri/src/lib.rs
git commit -m "L'exécutable sait dire où il range ses réglages"
```

---

### Tâche 3 : les trois appelants lisent au lieu de redemander

**Fichiers :**
- Modifier : `src-tauri/src/lib.rs` (bloc `.setup(…)`)
- Modifier : `src-tauri/src/commands.rs` (`fn config`, vers la ligne 2608)
- Modifier : `src-tauri/src/menu.rs` (`fn liste_recents`, vers la ligne 160)

**Interfaces :**
- Consomme : `emplacement::resoudre`, `emplacement::Emplacement` (tâche 1).
- Produit : `Emplacement` posé en état Tauri managé, lisible par
  `app.state::<crate::emplacement::Emplacement>()`.

- [ ] **Étape 1 : résoudre au `setup` et poser l'état**

Dans `src-tauri/src/lib.rs`, remplacer le corps du `.setup(|app| { … })` :

```rust
        .setup(|app| {
            use tauri::Manager;
            // Première ligne du démarrage, et elle doit le rester : le catalogue,
            // les préférences et les récents du menu descendent tous de cette racine,
            // et `providers()` initialiserait `PLATS` sur les seuls fournis si le
            // catalogue était chargé avant qu'elle ne soit connue.
            let emplacement = emplacement::resoudre(app.path().app_config_dir().ok());
            let refus = catalogue::initialiser(emplacement.racine.as_deref())
                .expect("le catalogue doit être chargé avant toute commande");
            app.manage(commands::CatalogueRefus(refus));
            // Avant `menu::poser`, qui lit déjà cet état pour bâtir les récents.
            app.manage(emplacement);
            menu::poser(app.handle())?;
            Ok(())
        })
```

L'ordre `manage` puis `poser` n'est pas cosmétique : `menu::poser` construit le
sous-menu des récents, donc lit l'état. Posé après, il lirait un état absent et
`state::<Emplacement>()` paniquerait.

- [ ] **Étape 2 : `commands.rs::config`**

Remplacer :

```rust
/// Répertoire de configuration de l'application, s'il est atteignable.
fn config(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok()
}
```

par :

```rust
/// Répertoire de configuration de l'application, s'il est atteignable.
///
/// Résolu une fois au démarrage par `emplacement::resoudre`, et relu ici : selon qu'un
/// marqueur voisine l'exécutable, c'est celui du système ou le `donnees/` de l'archive
/// portable. Ce fichier n'a pas à savoir lequel.
fn config(app: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    app.state::<crate::emplacement::Emplacement>().racine.clone()
}
```

Si `use tauri::Manager;` est déjà importé en tête de `commands.rs`, retirer la ligne
locale — `cargo clippy -D warnings` refuse l'import redondant.

- [ ] **Étape 3 : `menu.rs::liste_recents`**

Remplacer :

```rust
fn liste_recents(app: &AppHandle) -> Vec<String> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| preferences::charger(&d).recents_existants())
        .unwrap_or_default()
}
```

par :

```rust
fn liste_recents(app: &AppHandle) -> Vec<String> {
    use tauri::Manager;
    app.state::<crate::emplacement::Emplacement>()
        .racine
        .as_deref()
        .map(|d| preferences::charger(d).recents_existants())
        .unwrap_or_default()
}
```

Vérifier ensuite si `app.path()` reste utilisé dans `menu.rs` ; sinon, l'import
`tauri::Manager` en tête du fichier peut devenir inutile ou redondant — c'est clippy qui
tranche.

- [ ] **Étape 4 : la vérification qui tient tout le chantier**

```bash
grep -rn "app_config_dir" src-tauri/src | wc -l
```

Attendu : **exactement 1**, et cette occurrence est dans le `setup` de `lib.rs`.

```bash
grep -rn "app_config_dir" src-tauri/src
```

Attendu : `src-tauri/src/lib.rs:` … `emplacement::resoudre(app.path().app_config_dir().ok())`.

C'est la propriété qui empêche un quatrième appelant d'ouvrir un chemin parallèle sans
qu'on le voie. Si le compte est supérieur à 1, la tâche n'est pas finie.

- [ ] **Étape 5 : le témoin de non-régression**

```bash
cd src-tauri && cargo run --example temoin
```

Attendu : le **même compte de pages** que sur `main` avant la tâche. Aucun code de
composition n'a bougé.

Devant un écart du type `left: 18.75, right: 18.8`, ne pas conclure à une régression :
c'est la signature du piège documenté au CLAUDE.md. Relancer après

```bash
touch src-tauri/pods src-tauri/maquettes src-tauri/src/lib.rs
```

avant tout diagnostic.

- [ ] **Étape 6 : voir l'application démarrer**

```bash
cd src-tauri && cargo run
```

Attendu : la fenêtre s'ouvre, le menu **Fichier → Projets récents** est peuplé comme
avant, et l'étape Livraison liste les imprimeurs. Un panic au démarrage signale un
`manage` posé après son lecteur (étape 1).

- [ ] **Étape 7 : contrôles et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test --lib
cd .. && node --test tests/*.test.js
git add src-tauri/src/lib.rs src-tauri/src/commands.rs src-tauri/src/menu.rs
git commit -m "Un seul endroit demande encore son répertoire au système"
```

---

### Tâche 4 : le bandeau de lecture seule

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (une commande, près de `catalogue_refus`
  vers la ligne 282)
- Modifier : `src-tauri/src/lib.rs` (bloc `generate_handler!`)
- Modifier : `src/index.html` (section `#accueil`, vers la ligne 43)
- Modifier : `src/app.js` (`afficherAucunProjet`, vers la ligne 977)
- Modifier : `tests/contrats.test.js` (le faux `invoke` ligne 49, et deux tests)

**Interfaces :**
- Consomme : `emplacement::Mode` et l'état managé (tâches 1 et 3).
- Produit : la commande `emplacement_mode`, qui rend `"installe"`, `"portable"` ou
  `"portable-lecture-seule"` ; l'élément `#reglagesLectureSeule`.

- [ ] **Étape 1 : écrire les deux tests front**

Dans `tests/contrats.test.js`, à la suite des tests de `refusCatalogue` (vers la ligne
675) :

```js
/**
 * Une archive portable dépliée sur un support qui refuse l'écriture doit le dire, et le
 * dire à l'accueil. Sans ce mot, l'utilisateur travaille une heure et perd ses
 * maquettes à la fermeture — sans jamais savoir qu'elles n'ont pas été écrites.
 */
test('le mode portable en lecture seule s\'annonce à l\'accueil', async () => {
  const bloque = async (cmd, args) =>
    cmd === 'emplacement_mode' ? 'portable-lecture-seule' : invoke(cmd, args);
  const { els } = await charge({ invoke: bloque, open: async () => null });
  const p = els.get('reglagesLectureSeule');
  assert.equal(p.hidden, false);
  // Le dossier est nommé : c'est la seule information qui permette d'agir.
  assert.match(p.textContent, /donnees/);
});

/**
 * Les deux autres modes ne disent rien. Une application qui fonctionne n'a pas à
 * s'expliquer — et un bandeau permanent en mode portable serait relu par personne.
 */
test('les modes qui enregistrent restent muets', async () => {
  for (const mode of ['installe', 'portable']) {
    const dit = async (cmd, args) =>
      cmd === 'emplacement_mode' ? mode : invoke(cmd, args);
    const { els } = await charge({ invoke: dit, open: async () => null });
    assert.equal(els.get('reglagesLectureSeule').hidden, true, `mode ${mode}`);
  }
});
```

Et, dans le faux `invoke` de la ligne 49, à la suite de `if (cmd === 'catalogue_refus')
return [];` :

```js
  if (cmd === 'emplacement_mode') return 'installe';
```

- [ ] **Étape 2 : voir l'échec**

```bash
node --test tests/contrats.test.js
```

Attendu : les deux tests neufs échouent — `els.get('reglagesLectureSeule')` rend
`undefined`, donc `Cannot read properties of undefined (reading 'hidden')`. Le test
*existant* « chaque commande appelée par le front est déclarée au Rust » reste vert
pour l'instant : rien n'appelle encore la commande.

- [ ] **Étape 3 : la commande Rust**

Dans `src-tauri/src/commands.rs`, juste après `pub fn catalogue_refus` :

```rust
/// Sous quel régime l'application écrit ce qui n'appartient pas à un livre.
///
/// L'accueil ne s'en sert que pour un cas — la lecture seule —, mais la commande rend
/// les trois : un front qui n'aurait que le booléen « ça bloque » ne pourrait pas
/// distinguer une archive portable d'une installation, et c'est la question suivante
/// que pose quiconque cherche ses maquettes.
#[tauri::command]
pub fn emplacement_mode(e: State<crate::emplacement::Emplacement>) -> crate::emplacement::Mode {
    e.mode
}
```

Et la déclarer dans `src-tauri/src/lib.rs`, dans `generate_handler![…]`, à la suite de
`commands::catalogue_refus,` :

```rust
            commands::emplacement_mode,
```

- [ ] **Étape 4 : le bandeau dans `index.html`**

Dans `<section id="accueil">`, à la suite de `<div id="recents" class="recents"></div>`
et **dans** le même `<div class="bloc">` :

```html
      <!-- Version portable sur un support qui n'accepte pas l'écriture. À l'accueil et
           non à une étape : l'avertissement vaut pour tout ce qui suit, et c'est l'écran
           où l'on arrive. Le texte est ici et non dans le JS — il ne varie pas, et
           `app.js` n'a qu'à lever ou baisser le `hidden`. -->
      <p id="reglagesLectureSeule" class="note alerte" hidden>Version portable en
        lecture seule : le dossier « donnees », à côté de l'application, n'accepte pas
        l'écriture. Rien ne sera enregistré — ni les projets récents, ni les maquettes,
        ni les réglages de diffusion. Déplier l'archive sur un support inscriptible
        pour les retrouver.</p>
```

- [ ] **Étape 5 : le câblage dans `app.js`**

Ajouter la fonction à la suite de `afficherRecents` :

```js
/**
 * Le seul mot que l'emplacement adresse à l'utilisateur, et il ne le dit que sous
 * contrainte : le texte vit dans `index.html`, ici on ne fait que le montrer.
 */
async function afficherEmplacement() {
  const mode = await invoke('emplacement_mode');
  $('reglagesLectureSeule').hidden = mode !== 'portable-lecture-seule';
}
```

et l'appeler dans `afficherAucunProjet`, juste après `await afficherRecents();` :

```js
  await afficherRecents();
  await afficherEmplacement();
  majPied();
```

- [ ] **Étape 6 : voir passer**

```bash
node --test tests/*.test.js
```

Attendu : tout vert, y compris « chaque commande appelée par le front est déclarée au
Rust » — qui vérifie désormais que `emplacement_mode` figure bien dans
`generate_handler!`. S'il échoue, c'est l'étape 3 qui est incomplète.

```bash
cd src-tauri && cargo test --lib
```

Attendu : tout vert.

- [ ] **Étape 7 : le voir à l'écran**

```bash
cd src-tauri && touch src/lib.rs && cargo build
touch target/debug/ozalid-studio.portable
rm -rf target/debug/donnees       # la tâche 2 a pu en laisser un
: > target/debug/donnees          # un fichier là où un dossier est attendu
cargo run
```

Attendu : la fenêtre s'ouvre sur l'accueil, et le bandeau rouge s'y lit. Ouvrir les
devtools pour lire une erreur éventuelle plutôt que de la deviner.

Nettoyage :

```bash
cd src-tauri && rm target/debug/ozalid-studio.portable target/debug/donnees
```

- [ ] **Étape 8 : contrôles et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd .. && node --test tests/*.test.js
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src/index.html src/app.js tests/contrats.test.js
git commit -m "L'accueil prévient quand rien ne sera enregistré"
```

---

### Tâche 5 : `outils/portable.sh`

**Fichiers :**
- Créer : `outils/portable.sh` (exécutable)

**Interfaces :**
- Consomme : rien du code Rust ; lit `src-tauri/tauri.conf.json`,
  `src-tauri/target/release/`, `src-tauri/binaries/`, `src-tauri/fonts/`.
- Produit : `src-tauri/target/portable/ozalid-studio_<version>_x64-portable.zip`, dont
  la tâche 6 dépend. Le script imprime le chemin de l'archive sur sa dernière ligne.

- [ ] **Étape 1 : écrire le script**

```bash
#!/usr/bin/env bash
# Assemble l'archive portable pour Windows.
#
# Tauri 2 n'offre sous Windows que les cibles `nsis` et `msi` : il n'y a pas de cible
# « portable » à demander à `tauri build`, l'archive se monte à la main. Ici et non dans
# le workflow, pour la même raison que `typst.sh` et `polices.sh` : ce qui n'existe que
# dans la CI ne se vérifie que dans la CI, et personne n'ouvrirait le portable avant le
# premier utilisateur.
set -euo pipefail

ICI="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRIPLE="x86_64-pc-windows-msvc"
EXE="$ICI/src-tauri/target/release/ozalid-studio.exe"
SIDECAR="$ICI/src-tauri/binaries/typst-${TRIPLE}.exe"
POLICES="$ICI/src-tauri/fonts"
SORTIE="$ICI/src-tauri/target/portable"

for f in "$EXE" "$SIDECAR"; do
  [ -f "$f" ] || { echo "absent : $f" >&2; exit 1; }
done
ls "$POLICES"/*.ttf >/dev/null 2>&1 || { echo "aucune police dans $POLICES" >&2; exit 1; }

# La même source de vérité que le contrôle de tag du job `publier` : la version de
# l'application est celle de tauri.conf.json, et rien d'autre ne la porte.
VERSION="$(node -p "require('$ICI/src-tauri/tauri.conf.json').version")"
NOM="Ozalid Studio $VERSION"

rm -rf "$SORTIE"
mkdir -p "$SORTIE/$NOM"
cp "$EXE" "$SORTIE/$NOM/ozalid-studio.exe"
# Renommé, comme le bundle Tauri le fait d'un `externalBin` : `commands.rs::binaire_typst`
# cherche « typst.exe » à côté de l'exécutable, pas le nom triplé.
cp "$SIDECAR" "$SORTIE/$NOM/typst.exe"
mkdir -p "$SORTIE/$NOM/fonts"
cp "$POLICES"/* "$SORTIE/$NOM/fonts/"
# Le marqueur : vide, nommé d'après l'exécutable. C'est lui, et lui seul, qui fait
# écrire les réglages dans le « donnees/ » voisin plutôt que sur la machine hôte.
# `donnees/` n'est pas livré : il naît au premier lancement, et une archive qui le
# contiendrait ferait croire à un dossier qu'on peut écraser en mettant à jour.
: > "$SORTIE/$NOM/ozalid-studio.portable"

ARCHIVE="$SORTIE/ozalid-studio_${VERSION}_x64-portable.zip"
# `tar` et non `zip` : `zip` n'est pas garanti dans le Git Bash des runners Windows,
# où ce script tourne — la même raison qui fait passer `typst.sh` par `tar` pour
# l'extraction. Le `-a` prend le format sur l'extension, et deflate.
(cd "$SORTIE" && tar -a -cf "$ARCHIVE" "$NOM")

echo "$ARCHIVE"
```

```bash
chmod +x outils/portable.sh
```

- [ ] **Étape 2 : le faire échouer d'abord**

Depuis un dépôt qui n'a pas de binaire Windows — c'est le cas sur un poste macOS :

```bash
outils/portable.sh
```

Attendu : `absent : …/src-tauri/target/release/ozalid-studio.exe`, code de sortie 1. Un
script qui rendrait 0 sur une arborescence vide produirait une archive vide, et la CI
la publierait.

- [ ] **Étape 3 : le faire réussir sur des leurres**

Le montage se vérifie sans compiler pour Windows :

```bash
mkdir -p src-tauri/target/release
: > src-tauri/target/release/ozalid-studio.exe
outils/portable.sh
```

Attendu : le chemin de l'archive sur la dernière ligne. Puis en vérifier le contenu :

```bash
tar -tf "$(outils/portable.sh)"
```

Attendu, sous un unique dossier de tête `Ozalid Studio <version>/` :
`ozalid-studio.exe`, `ozalid-studio.portable`, `typst.exe`, `fonts/` avec ses `.ttf`, et
**pas** de `donnees/`.

Nettoyage du leurre :

```bash
rm src-tauri/target/release/ozalid-studio.exe
```

- [ ] **Étape 4 : commit**

```bash
git add outils/portable.sh
git commit -m "Une archive se monte à la main, faute de cible portable chez Tauri"
```

---

### Tâche 6 : la CI produit l'archive, et l'interroge

**Fichiers :**
- Modifier : `.github/workflows/windows.yml`, job `publier` uniquement

**Interfaces :**
- Consomme : `outils/portable.sh` (tâche 5) et le drapeau `--emplacement` (tâche 2).
- Produit : l'archive jointe à la release draft.

- [ ] **Étape 1 : ajouter les deux étapes**

Dans le job `publier`, **entre** l'étape « Vérifier l'installation silencieuse » et
l'étape « Release draft » :

```yaml
      - name: Construire l'archive portable
        run: outils/portable.sh

      # Une archive qui se déplie n'est pas une archive qui range ses réglages à côté
      # d'elle. On interroge le binaire réellement livré, par son propre code, plutôt
      # que de faire confiance à un test qui simulerait la même logique ailleurs.
      #
      # `--emplacement` écrit dans un fichier et non sur la sortie standard : l'exécutable
      # est compilé en `windows_subsystem = "windows"`, il n'a aucune console rattachée
      # et un `println!` ne serait lu par personne.
      - name: Vérifier que l'archive est portable
        shell: pwsh
        timeout-minutes: 5
        run: |
          $zip = Get-ChildItem "src-tauri/target/portable" -Filter "*-portable.zip" |
            Select-Object -First 1
          if (-not $zip) { Write-Error "aucune archive portable produite"; exit 1 }

          $depliage = Join-Path $env:RUNNER_TEMP "portable"
          Expand-Archive -Path $zip.FullName -DestinationPath $depliage -Force
          $racine = (Get-ChildItem $depliage -Directory | Select-Object -First 1).FullName
          Write-Host "déplié dans $racine"
          Get-ChildItem $racine -Recurse -Depth 1 | ForEach-Object { $_.FullName }

          # Les deux mêmes contrôles que sur l'installation silencieuse : le sidecar à
          # côté de l'exécutable, les polices dans leur sous-dossier.
          if (-not (Test-Path (Join-Path $racine "typst.exe"))) {
            Write-Error "typst.exe absent de $racine"
            exit 1
          }
          if (-not (Get-ChildItem (Join-Path $racine "fonts") -Filter "*.ttf" -ErrorAction SilentlyContinue)) {
            Write-Error "aucun .ttf dans $racine\fonts"
            exit 1
          }

          $rapport = Join-Path $env:RUNNER_TEMP "emplacement.txt"
          $p = Start-Process -FilePath (Join-Path $racine "ozalid-studio.exe") `
            -ArgumentList "--emplacement", $rapport -Wait -PassThru
          if ($p.ExitCode -ne 0) {
            Write-Error "le drapeau --emplacement a rendu $($p.ExitCode)"
            exit 1
          }
          $lu = Get-Content $rapport -Raw
          Write-Host $lu

          $attendue = Join-Path $racine "donnees"
          # La ligne entière, ancrée. « portable\b » passerait sur
          # « mode = portable-lecture-seule » — le tiret est une frontière de mot — et la
          # CI validerait une archive qui n'enregistre rien.
          if ($lu -notmatch "(?m)^mode = portable$") {
            Write-Error "l'archive ne se reconnaît pas portable : $lu"
            exit 1
          }
          if ($lu -notmatch [regex]::Escape($attendue)) {
            Write-Error "racine attendue « $attendue », rapport : $lu"
            exit 1
          }
          Write-Host "l'archive range ses réglages dans $attendue"
```

- [ ] **Étape 2 : joindre l'archive à la release**

Remplacer le corps de l'étape « Release draft » :

```yaml
      - name: Release draft
        env:
          GH_TOKEN: ${{ github.token }}
        run: |
          exe=$(find src-tauri/target/release/bundle/nsis -name "*-setup.exe" | head -1)
          [ -n "$exe" ] || { echo "aucun installeur produit" >&2; exit 1; }
          zip=$(find src-tauri/target/portable -name "*-portable.zip" | head -1)
          [ -n "$zip" ] || { echo "aucune archive portable produite" >&2; exit 1; }
          gh release create "$GITHUB_REF_NAME" "$exe" "$zip" \
            --draft \
            --title "Ozalid Studio $GITHUB_REF_NAME" \
            --notes "Deux façons de l'installer.

          **L'installeur \`.exe\`** — la voie normale. L'installation ne demande pas de droits administrateur.

          **L'archive \`-portable.zip\`** — à déplier où l'on veut, y compris sur une clé USB. Elle n'installe rien et garde ses réglages dans le dossier « donnees » qu'elle crée à côté d'elle ; deux postes différents y retrouvent les mêmes maquettes. Deux réserves : elle ne peut pas installer le composant **WebView2** dont l'application a besoin — présent d'origine sur Windows 11 et sur un Windows 10 à jour, à défaut de quoi il faut passer par l'installeur — et Windows marque les fichiers extraits d'une archive téléchargée : faire « Propriétés » puis « Débloquer » sur le zip **avant** de l'extraire.

          Rien n'est signé : au premier lancement, Windows affiche « Windows a protégé votre PC ». Choisir « Informations complémentaires », puis « Exécuter quand même »."
```

- [ ] **Étape 3 : vérifier la syntaxe du workflow**

```bash
python3 -c "
import yaml
etapes = yaml.safe_load(open('.github/workflows/windows.yml'))['jobs']['publier']['steps']
for e in etapes: print(e.get('name', '(sans nom)'))
"
```

Attendu : le YAML se charge sans erreur, et la liste contient, dans cet ordre,
« Vérifier l'installation silencieuse », « Construire l'archive portable », « Vérifier
que l'archive est portable », « Release draft ».

Si `pyyaml` n'est pas installé (`pip install pyyaml`, ou l'exécuter dans un
environnement qui l'a), relire le fichier à l'œil : l'indentation d'un bloc `run: |` est
la seule erreur probable, et une erreur de YAML ne se découvrirait sinon qu'au tag.

- [ ] **Étape 4 : commit**

```bash
git add .github/workflows/windows.yml
git commit -m "La CI déplie l'archive et lui demande où elle range ses réglages"
```

- [ ] **Étape 5 : la preuve, qui n'arrive qu'au tag**

Ce job ne tourne que sur `refs/tags/v*`. Le plan ne le déclenche pas : poser un tag est
une décision de publication, pas une étape d'implémentation. La vérification réelle a
lieu au premier tag posé après ce chantier, et c'est à ce moment-là qu'il faut lire le
journal de l'étape « Vérifier que l'archive est portable ».

**Ne pas déclarer le chantier vérifié avant cette lecture.** Ce que les tâches 1 à 5
prouvent, c'est que le code résout, que le script monte et que le front avertit ; que
l'archive livrée soit portable sur un vrai Windows, seul ce job le dira.

---

### Tâche 7 : la documentation

**Fichiers :**
- Modifier : `README.md`, section **Windows** (lignes 38 à 46)

**Interfaces :** aucune.

Le README ne dit nulle part où vont les réglages : `%LOCALAPPDATA%\Ozalid Studio` y
désigne le lieu d'**installation**. Il n'y a donc aucune phrase à corriger, seulement
un manque à combler — pour les deux modes à la fois.

- [ ] **Étape 1 : insérer la section portable**

Dans `README.md`, entre le paragraphe de l'installeur (qui se termine par
`` `%LOCALAPPDATA%\Ozalid Studio`. ``) et le paragraphe SmartScreen (qui commence par
« Au premier lancement »), insérer :

```markdown
Une **archive portable** (`-portable.zip`) est publiée à côté de l'installeur. On la
déplie où l'on veut — un disque, un dossier partagé, une clé USB —, elle n'installe rien
et ne laisse rien sur la machine : elle garde ses réglages dans le dossier `donnees`
qu'elle crée à côté de l'exécutable, et la même clé rouvre ses maquettes sur un autre
poste. La version installée, elle, les range là où le système les met.

Ce qui l'en distingue tient à un fichier vide, `ozalid-studio.portable`, livré dans
l'archive : c'est sa seule présence qui fait descendre les réglages dans `donnees`. Le
supprimer rend le comportement de la version installée.

Deux réserves, dans cet ordre d'importance :

- L'archive ne peut pas installer le composant **WebView2**, dont l'application a besoin
  pour afficher son interface. Il est présent d'origine sur Windows 11 et sur un
  Windows 10 à jour ; sur un poste plus ancien, l'application ne s'ouvrira pas, et il
  faut passer par l'installeur, qui sait le télécharger.
- Windows marque les fichiers extraits d'une archive téléchargée. Faire **Propriétés**
  puis **Débloquer** sur le `.zip` *avant* de l'extraire, sans quoi le lancement peut
  être bloqué sans explication.

Et une limite connue : les projets récents sont mémorisés en chemins absolus. Sur une
clé USB dont la lettre de lecteur change d'un poste à l'autre, la liste se vide — rien
n'est perdu, elle se repeuple au premier projet rouvert.

Pour savoir où une copie donnée range ses réglages, sans ouvrir l'application :

    ozalid-studio.exe --emplacement rapport.txt

écrit le mode et le chemin dans `rapport.txt`, puis s'arrête. Dans un fichier et non à
l'écran : l'exécutable est compilé sans console.
```

Le paragraphe SmartScreen qui suit vaut pour les deux formes de livraison — rien n'est
signé —, et sa place après les deux le dit sans qu'on ait à l'écrire.

- [ ] **Étape 2 : relire l'ensemble**

```bash
grep -n -i "portable\|LOCALAPPDATA\|WebView2\|donnees" README.md
```

Attendu : la section Windows nomme les deux formes de livraison, et aucune phrase
n'affirme que les réglages sont *toujours* au même endroit.

- [ ] **Étape 3 : commit**

```bash
git add README.md
git commit -m "Le README dit les deux façons d'installer, et ce qu'elles coûtent"
```

---

## Ce que ce plan ne prouve pas

À faire une fois, à la main, sur un vrai Windows et une vraie clé — la CI n'ouvre
jamais la fenêtre :

- déplier l'archive sur une clé USB, lancer, enregistrer une maquette, vérifier qu'elle
  est sous `donnees/maquettes/` et **nulle part dans `%APPDATA%`** ;
- débrancher, rebrancher sur un autre poste, retrouver la maquette ;
- déplier sur un support en lecture seule et lire le bandeau de l'accueil ;
- **composer un livre depuis l'archive** — le sidecar et les polices sont censés suivre,
  et c'est le seul endroit où on le verra vraiment.

Tant que ces quatre points ne sont pas faits, le chantier est implémenté, pas vérifié.
