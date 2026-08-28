//! Compose le manuscrit-témoin et vérifie que la pagination n'a pas bougé.
//!
//! Le témoin est *Candide* (Voltaire, 1759), du domaine public, récupéré depuis Project
//! Gutenberg et mis au format du projet. Il n'est pas là pour se lire : `build/` n'étant
//! pas versionné, c'est le seul livre que l'intégration continue puisse composer.
//!
//! Ce qu'il prouve, et qu'aucun test unitaire ne peut prouver : Typst compose le même
//! nombre de pages sur macOS et sur Windows. Un écart invaliderait la promesse centrale
//! du projet — un dos calculé sur une plateforme ne vaudrait que pour elle.
//!
//! L'imprimeur est BoD, et non Lulu : la table Lulu ne porte pas de tranche de gouttière
//! sous 151 pages, et la compléter pour les besoins d'un test reviendrait à laisser le
//! test dicter la production.
//!
//! **Deux témoins, et non un.** Le premier a longtemps été seul, et ne couvrait alors
//! qu'un format sur les dix que BoD publie et un papier sur quatre : les neuf formats
//! ouverts au lot 4 n'étaient exercés par aucune composition, et la revue finale a dû les
//! composer à la main pour les regarder. Le second prend l'autre bout de la table — le
//! plus petit format, sur le papier le plus épais — parce que c'est là que la pagination
//! monte, que la gouttière change de tranche et que le plafond du papier se rapproche.
//! Il coûte une composition de plus, et c'est tout ce qu'il coûte.
//!
//! Usage : cargo run --example temoin [répertoire de sortie]

use std::path::{Path, PathBuf};

use ozalid_lib::catalogue;
use ozalid_lib::maquettes;
use ozalid_lib::package;
use ozalid_lib::planche::Releve;
use ozalid_lib::projet::{Livre, Projet};
use ozalid_lib::typst::Typst;

/// Les fabrications composées, et la pagination attendue de chacune.
///
/// Chaque pagination est **relevée**, sur macOS avec Typst 0.15.1 et EB Garamond, au
/// corps et à l'interligne que `interieur` fixe pour tout gabarit. Elle dépend de chacun
/// de ces éléments : la déplacer est un acte délibéré, à revalider sur un livre réel —
/// jamais un ajustement pour faire passer l'intégration continue.
const TEMOINS: &[(&str, &str, &str, &str, u32)] = &[
    ("bod", "135x215", "broche", "creme-90", 98),
    ("bod", "120x190", "broche", "photo-brillant-130", 118),
];

fn main() -> Result<(), String> {
    let sortie = std::env::args()
        .nth(1)
        .map_or_else(|| std::env::temp_dir().join("ozalid-temoin"), PathBuf::from);

    let livre = Livre {
        isbn: String::new(),
        titre: "Candide".into(),
        // Le jeton, comme un projet neuf : la page de titre reprend le titre.
        titre_page: "%TITRE%".into(),
        auteur: "Voltaire".into(),
        genre: "conte philosophique".into(),
        editeur: "Editeur".into(),
        collection: "Collection".into(),
        monogramme: "Monogramme".into(),
        copyright: "Texte du domaine public.".into(),
        prix: "Prix".into(),
        mention: "Mention".into(),
        // Sans dédicace, délibérément : c'est ce qui garde le témoin à 98 pages.
        dedicace: String::new(),
        chapitres: Some(30),
    };
    let mut projet = Projet::nouveau(livre, include_str!("../temoin/manuscrit.md").to_string());
    // La Filets est purement typographique : le témoin traverse la planche entière sans
    // qu'une seule image ait à être versionnée.
    projet.meta.couverture.maquette = Some(
        maquettes::par_cle(None, "filets")
            .expect("maquette fournie « blanche »")
            .couverture,
    );

    let typst =
        Typst::new("typst").avec_polices(Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"));

    // Chacun jusqu'au bout, même après un écart : deux paginations déplacées d'un coup
    // disent « la version de Typst a bougé » là où une seule dit « ce gabarit-là ». Le
    // premier écart rencontré ne doit donc pas masquer le second.
    let mut ecarts = Vec::new();
    for &(pod, format, reliure, papier, attendues) in TEMOINS {
        match compose(&projet, &typst, &sortie, (pod, format, reliure, papier)) {
            Ok(pages) if pages != attendues => ecarts.push(format!(
                "{pod}-{format}-{reliure}-{papier} : {pages} pages, {attendues} attendues"
            )),
            Ok(_) => {}
            Err(e) => ecarts.push(format!("{pod}-{format}-{reliure}-{papier} : {e}")),
        }
    }
    if !ecarts.is_empty() {
        return Err(format!(
            "pagination déplacée —\n  {}\n\
             Si le changement est voulu — police, gabarit, version de Typst —, relever la \
             nouvelle valeur et la figer dans TEMOINS. Sinon, cette plateforme ne compose \
             pas comme l'autre, et aucun dos calculé ici ne vaut ailleurs.",
            ecarts.join("\n  ")
        ));
    }
    Ok(())
}

/// Compose un livrable jusqu'à sa planche, et rend sa pagination.
///
/// La planche entière, et pas seulement l'intérieur : c'est elle qui exerce la formule
/// de dos du papier et le fond perdu du format, et une planche qui ne s'assemble pas est
/// un écart au même titre qu'une page de trop.
fn compose(
    projet: &Projet,
    typst: &Typst,
    sortie: &Path,
    (pod, format, reliure, papier): (&str, &str, &str, &str),
) -> Result<u32, String> {
    let r = catalogue::resout(&catalogue::Fabrication {
        pod: pod.into(),
        format: format.into(),
        reliure: reliure.into(),
        papier: papier.into(),
    })?;
    let pr = r.provider();
    // Sous la clé du **gabarit**, comme le package : deux témoins sur des formats
    // différents ne se marchent pas dessus, deux papiers d'un même gabarit partageraient
    // leur intérieur.
    let int = package::composer_interieur(projet, &pr, &pr.cle, sortie, typst)?;
    let p = package::assembler(
        projet,
        &pr,
        r.papier,
        // BoD publie son dos et son fond perdu : le relevé est ignoré.
        Releve::default(),
        &pr.cle,
        // Le témoin mesure une pagination, il ne prépare pas une commande : aucune
        // finition à déclarer.
        None,
        &int,
        sortie,
        typst,
    )?;

    println!(
        "{} en {} — {} pages, gouttière {:.1} mm, dos {:.2} mm, planche {:.2} × {:.2} mm{}",
        p.libelle,
        p.papier,
        p.pages,
        p.gouttiere,
        p.dos,
        p.planche.0,
        p.planche.1,
        if p.blanche {
            ", blanche de parité"
        } else {
            ""
        }
    );
    Ok(p.pages)
}
