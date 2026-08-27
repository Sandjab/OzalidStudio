//! `initialiser` dans un processus qui n'appartient qu'à lui.
//!
//! `PLATS` est un `OnceLock` de processus, et la suite `--lib` en partage un seul où des
//! dizaines de tests appellent `provider(…)` : `initialiser` y serait toujours précédé,
//! et un test le concernant passerait ou tomberait selon l'ordre d'exécution. Ici rien ne
//! le précède.
//!
//! **Un seul `#[test]` dans ce fichier**, pour la même raison : les tests d'un binaire
//! d'intégration partagent son processus, donc son `OnceLock`. Un second échouerait selon
//! l'ordre. Ce qu'il faudrait tester en plus demande un fichier de plus.

use std::io::Write;

use ozalid_lib::catalogue;

const IMPRIMEUR_ESSAI: &str = r##"
cle = "essai"
nom = "Imprimeur d'essai"
fond_perdu = 4.0

[[format]]
cle = "100x150"
nom = "10 × 15 cm"
mm = { largeur = 100.0, hauteur = 150.0 }
marges = { haut = 10.0, bas = 10.0, exterieur = 10.0 }
gouttieres = [ { de = 24, a = 400, mm = 15.0 } ]

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 400 }
parite = "paire"

[[papier]]
cle = "standard"
nom = "Papier standard"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.06, plus = 0.0 }
"##;

/// Les trois choses que le démarrage promet, et que `--lib` ne peut pas dire : que le
/// chargement a lieu, que la vue plate porte réellement le fichier du poste, et qu'un
/// second appel se refuse tout haut plutôt que d'ignorer ce fichier en silence.
#[test]
fn le_demarrage_charge_les_fichiers_du_poste_et_refuse_un_second_chargement() {
    let d = tempfile::TempDir::new().unwrap();
    let pods = d.path().join("pods");
    std::fs::create_dir_all(&pods).unwrap();
    let mut f = std::fs::File::create(pods.join("essai.toml")).unwrap();
    f.write_all(IMPRIMEUR_ESSAI.as_bytes()).unwrap();

    let refus = catalogue::initialiser(Some(d.path())).expect("le premier chargement");
    assert!(refus.is_empty(), "{refus:?}");

    // Toute la promesse du chantier tient dans cette ligne : un imprimeur que le binaire
    // ne connaît pas est servi comme les autres, sous la clé de son gabarit.
    let essai = catalogue::providers()
        .iter()
        .find(|p| p.cle == "essai-100x150-broche")
        .expect("le POD du poste n'est pas dans la vue plate");
    assert_eq!(essai.format, (100.0, 150.0));
    assert_eq!(
        catalogue::providers().len(),
        24,
        "vingt-trois fournis, plus le déposé"
    );

    // `providers()` n'est qu'une projection : la preuve que le poste a vraiment atteint
    // `PODS`, et non seulement sa vue plate, se fait sur `pods()` lui-même.
    assert!(
        catalogue::pods().iter().any(|p| p.cle == "essai"),
        "le POD du poste n'est pas dans PODS"
    );

    // `resout` interroge `PODS`, que `providers()` — donc l'assertion ci-dessus — n'initialise
    // pas : oublier de le remplir dans `initialiser` resterait vert sans cette preuve-ci.
    let r = catalogue::resout(&catalogue::Fabrication {
        pod: "essai".into(),
        format: "100x150".into(),
        reliure: "broche".into(),
        papier: "standard".into(),
    })
    .expect("le POD du poste n'est pas dans PODS");
    assert_eq!(r.provider().format, (100.0, 150.0));

    // Un second appel est un défaut d'ordonnancement : il doit s'entendre, sans quoi les
    // fichiers du poste seraient ignorés en silence.
    assert_eq!(
        catalogue::initialiser(Some(d.path())).unwrap_err(),
        "le catalogue a déjà été chargé"
    );
}
