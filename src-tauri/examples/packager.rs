//! Produit les packages d'un projet pour un ou plusieurs livrables.
//!
//! C'est la chaîne entière en une commande : intérieur composé, pagination mesurée,
//! dos calculé, planche assemblée. Sans interface, donc utilisable pour vérifier que
//! Typst compile ce que le moteur émet — ce qu'aucun test unitaire ne peut faire.
//!
//! Usage : cargo run --example packager -- <projet.ozalid> <sortie> <pod> <format> <reliure>…

use std::path::{Path, PathBuf};

use ozalid_lib::catalogue;
use ozalid_lib::package;
use ozalid_lib::planche::Releve;
use ozalid_lib::projet::Projet;
use ozalid_lib::typst::Typst;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 5 || !(args.len() - 2).is_multiple_of(3) {
        eprintln!(
            "usage : packager <projet.ozalid> <répertoire de sortie> \
             <pod> <format> <reliure>…"
        );
        std::process::exit(2);
    }
    let projet = Projet::ouvrir(Path::new(&args[0]))?;
    let racine = PathBuf::from(&args[1]);
    let gabarits = &args[2..];
    let typst =
        Typst::new("typst").avec_polices(Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"));

    for triplet in gabarits.chunks_exact(3) {
        let [pod, format, reliure] = triplet else {
            unreachable!("chunks_exact(3)")
        };
        let papier = catalogue::pod(pod)
            .and_then(|p| p.papiers.first())
            .ok_or_else(|| format!("POD inconnu : {pod}"))?
            .cle
            .clone();
        let resolu = catalogue::resout(&catalogue::Fabrication {
            pod: pod.clone(),
            format: format.clone(),
            reliure: reliure.clone(),
            papier,
        })?;
        let pr = resolu.provider();
        // Un relevé de secours pour les imprimeurs à gabarit, afin que l'exemple
        // puisse les traverser aussi ; l'interface, elle, le demande à l'utilisateur.
        let releve = Releve {
            dos: Some(17.0),
            fond_perdu: Some(3.0),
        };
        let sortie = racine.join(&pr.cle);
        let cible = package::Cible {
            papier: resolu.papier.clone(),
            releve,
            // L'exemple ne déclare pas de finition : c'est une donnée de commande, et
            // elle vit sur le livrable du projet, pas sur un gabarit.
            finition: None,
            cle: pr.cle.clone(),
            pr,
        };
        let int = package::composer_interieur(&projet, &cible.pr, &cible.cle, &sortie, &typst)?;
        let p = package::assembler(&projet, &cible, &int, &sortie, &typst)?;
        println!(
            "{} — {} pages, gouttière {:.1} mm, dos {:.2} mm, planche {:.2} × {:.2} mm{}",
            p.libelle,
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
        // Ce que la composition a relevé sans échouer : les mêmes phrases qu'à
        // l'écran, pour que la chaîne en ligne de commande dise ce que l'interface dit.
        for a in &p.avertissements {
            println!("   ⚠ {a}");
        }
        for c in &p.chemins {
            println!("   {c}");
        }
    }
    Ok(())
}
