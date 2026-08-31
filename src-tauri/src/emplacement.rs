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
        assert!(
            d.path().join("donnees").is_dir(),
            "le dossier doit être créé"
        );
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

    /// L'argument est reconnu, et le chemin qui le suit est celui du rapport.
    #[test]
    fn le_drapeau_donne_le_fichier_de_sortie() {
        let args = ["--emplacement", "rapport.txt"].map(String::from);
        assert_eq!(sortie_demandee(args), Some(PathBuf::from("rapport.txt")));
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
}
