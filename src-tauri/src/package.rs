//! Le package d'un prestataire : l'intérieur, la planche, et de quoi les relire.
//!
//! Un livre, N prestataires, aucun réglage retouché entre les deux — c'est la « file
//! d'attente » du COOKBOOK, exécutée. Chaque prestataire coché déclenche sa propre
//! composition : son format, sa gouttière, sa pagination, donc son dos et sa planche.
//!
//! L'ordre des opérations n'est pas négociable : l'intérieur d'abord, parce que c'est
//! lui qui donne la pagination ; le dos ensuite, parce qu'il en découle ; la planche
//! enfin. Inverser reviendrait à ressaisir un nombre de pages à la main, ce que
//! l'application existe pour supprimer.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::catalogue::{Papier, Provider};
use crate::couverture::Ressource;
use crate::interieur::{self, Reglage};
use crate::manuscrit;
use crate::planche::{self, Gabarit, Releve};
use crate::projet::Projet;
use crate::typst::Typst;

/// Ce qu'un package contient une fois écrit sur le disque.
#[derive(Debug, Clone, Serialize)]
pub struct Package {
    pub cle: String,
    pub libelle: String,
    pub papier: String,
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
    pub dos: f64,
    /// Épaisseur que le texte du dos réclame, **quand elle dépasse `dos`** : le titre
    /// part rogné au pli sur ce PDF-là. `None`, il tient. C'est la seule chose qu'une
    /// maquette unique pour N formats casse au lieu de simplement la déplacer — le
    /// corps du dos suit la largeur de couverture, son épaisseur suit la pagination.
    pub dos_requis: Option<f64>,
    pub fond_perdu: f64,
    /// Dimensions de la planche, en mm.
    pub planche: (f64, f64),
    pub chemins: Vec<String>,
    /// La planche en PNG, à côté du PDF. Elle ne part pas chez l'imprimeur — d'où sa
    /// place hors de `chemins` : c'est de quoi vérifier d'un coup d'œil que la planche
    /// tient, pour ce prestataire-là, avec le dos qu'il a réellement mesuré.
    pub vignette: String,
    /// Familles que Typst n'a pas trouvées et a remplacées par une écriture de repli
    /// — sans échouer, donc sans que rien d'autre ne le dise. Vide, tout va bien.
    pub polices_introuvables: Vec<String>,
    /// Vrai quand l'intérieur de ce package est la copie de celui d'un autre livrable
    /// du même gabarit : il n'a pas été recomposé.
    pub interieur_partage: bool,
}

/// Nom de fichier des sorties d'un livrable. Le nom porte la clé entière : deux
/// packages ouverts côte à côte ne peuvent pas être confondus, deux papiers non plus.
pub(crate) fn nom(cle: &str, quoi: &str, ext: &str) -> String {
    format!("{quoi}-{cle}.{ext}")
}

/// L'intérieur composé d'un gabarit : ce que deux livrables du même gabarit partagent.
/// La planche, elle, reste par livrable — le dos suit le papier.
#[derive(Debug, Clone)]
pub struct InterieurCompose {
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
    pub polices_introuvables: Vec<String>,
    pub src: PathBuf,
    pub pdf: PathBuf,
}

/// Compose l'intérieur : la convergence, puis le PDF. C'est le bloc 1 de l'ancien
/// `assembler`, sorti pour n'être payé qu'une fois par gabarit.
pub fn composer_interieur(
    projet: &Projet,
    pr: &Provider,
    cle: &str,
    dossier: &Path,
    typst: &Typst,
) -> Result<InterieurCompose, String> {
    let int = &projet.meta.interieur;
    // `interieur::source` interpole la police sans échappement : la validation est ici.
    int.verifie()?;
    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", dossier.display()))?;
    let livre = &projet.meta.livre;
    let chapitres = manuscrit::decoupe(&projet.texte, livre.chapitres)?;

    let src = dossier.join(nom(cle, "interieur", "typ"));
    let r = interieur::converge(pr, |reglage| {
        ecrire(
            &src,
            &interieur::source(livre, int, pr, reglage, &chapitres, None),
        )?;
        typst.pages(&src)
    })?;
    let reglage = Reglage {
        gouttiere: r.gouttiere,
        blanche: r.blanche,
    };
    ecrire(
        &src,
        &interieur::source(livre, int, pr, &reglage, &chapitres, None),
    )?;
    let pdf = dossier.join(nom(cle, "interieur", "pdf"));
    let polices_introuvables = typst.compile(&src, &pdf)?;
    Ok(InterieurCompose {
        pages: r.pages,
        gouttiere: r.gouttiere,
        blanche: r.blanche,
        polices_introuvables,
        src,
        pdf,
    })
}

/// Refuse une pagination hors de ce que le livrable admet, en nommant qui la borne.
///
/// Hors d'`assembler` pour être testable : le contrôle y était inline, et aucun test ne
/// pouvait l'atteindre sans composer un intérieur. C'est le même arbitrage que
/// `verifie_pages`, dans ce fichier.
///
/// Le message nomme le **papier** quand la reliure seule aurait accepté ce compte de
/// pages : « hors des 24 à 900 que BoD accepte en broche » enverrait chercher l'erreur du
/// mauvais côté pour un livre de 880 pages en photo brillant. À l'inverse, un compte que
/// la reliure seule refuse déjà ne nomme pas le papier — changer de papier ne le sauverait
/// pas.
fn verifie_pagination(cle: &str, pages: u32, pr: &Provider, papier: &Papier) -> Result<(), String> {
    let (min, max) = papier.bornes_dans(pr.pages_min, pr.pages_max);
    if pages >= min && pages <= max {
        return Ok(());
    }
    // La vraie question n'est pas à qui appartient la borne affichée, mais si la reliure
    // seule aurait accepté ce compte : changer de papier ne sauve un livre que si c'est
    // le papier, et lui seul, qui l'a fait échouer.
    let du_papier = if pages < min {
        pages >= pr.pages_min
    } else {
        pages <= pr.pages_max
    };
    let en = if du_papier {
        format!("{} en {}", pr.fabrication.reliure, papier.nom)
    } else {
        pr.fabrication.reliure.clone()
    };
    Err(format!(
        "{cle} : {pages} pages, hors des {min} à {max} que {} accepte en {en}.",
        pr.libelle
    ))
}

/// Assemble un package : le dos, découlant de la pagination de l'intérieur, puis la
/// planche. L'intérieur lui-même n'est pas recomposé — il est reçu tout fait, composé
/// une fois par gabarit par `composer_interieur` (directement ou via `lot`), et copié
/// ici s'il vient d'un autre répertoire que celui de ce livrable.
///
/// Le `releve` ne sert que chez les prestataires qui ne publient ni dos ni fond perdu ;
/// ailleurs, il est ignoré au profit de leur formule.
// La signature retombera à six arguments à la tâche 5, quand `Provider` portera sa
// `fabrication` : `pr` seul suffira à ce que `cle` et `releve` disent aujourd'hui à
// part. `expect` et non `allow` : si le compte baisse avant que ce lint ne soit
// retiré, le build doit le dire.
#[expect(clippy::too_many_arguments)]
pub fn assembler(
    projet: &Projet,
    pr: &Provider,
    papier: &Papier,
    releve: Releve,
    cle: &str,
    interieur: &InterieurCompose,
    dossier: &Path,
    typst: &Typst,
) -> Result<Package, String> {
    let livre = &projet.meta.livre;
    std::fs::create_dir_all(dossier)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", dossier.display()))?;

    // 1. L'intérieur du gabarit, composé ailleurs ou ici : s'il vient d'un autre
    // répertoire, il est copié sous le nom de ce livrable — les octets sont les mêmes,
    // et c'est le sens de « une seule composition ».
    let src_int = dossier.join(nom(cle, "interieur", "typ"));
    let pdf_int = dossier.join(nom(cle, "interieur", "pdf"));
    let interieur_partage = interieur.pdf != pdf_int;
    if interieur_partage {
        for (de, vers) in [(&interieur.src, &src_int), (&interieur.pdf, &pdf_int)] {
            // Garde-fou : `InterieurCompose` est public à champs publics, rien
            // n'empêche un appelant de passer `dossier` égal à celui où l'intérieur a
            // déjà été composé. `fs::copy` sur un fichier vers lui-même le tronque sous
            // Unix — s'y refuser coûte moins qu'un dos recalculé sur un PDF vidé.
            if de != vers {
                std::fs::copy(de, vers)
                    .map_err(|e| format!("copie de l'intérieur ({}) : {e}", vers.display()))?;
            }
        }
    }
    // Ce contrôle tombe après la composition de l'intérieur du gabarit : un refus ici
    // coûte donc une composition — que la mémoïsation de `lot` ne repaie pas pour le
    // livrable suivant du même gabarit, qui la retentera à son tour.
    verifie_pagination(cle, interieur.pages, pr, papier)?;
    let mut polices_introuvables = interieur.polices_introuvables.clone();

    // 2. Le dos découle de cette pagination-là, jamais d'une saisie.
    let g = Gabarit::pour(pr, papier, interieur.pages, releve)?;

    // 3. La planche.
    let cv = projet
        .meta
        .couverture
        .maquette
        .as_ref()
        .ok_or("aucune maquette de couverture : en choisir une avant de packager.")?;
    let (une, quatre) = ecrire_images(projet, dossier)?;
    let src_pl = dossier.join(nom(cle, "couverture", "typ"));
    ecrire(
        &src_pl,
        &planche::source(livre, cv, &g, une.as_ref(), quatre.as_ref())?,
    )?;
    let pdf_pl = dossier.join(nom(cle, "couverture", "pdf"));
    // La planche a ses propres polices : ses substitutions s'ajoutent à celles de
    // l'intérieur, chaque famille une fois.
    for f in typst.compile(&src_pl, &pdf_pl)? {
        if !polices_introuvables.contains(&f) {
            polices_introuvables.push(f);
        }
    }

    // 4. La même planche en vignette, depuis la même source : ce qu'on regarde est ce
    // qui part à l'impression, et non une approximation qu'on espère fidèle. 72 ppp
    // suffisent à juger un débord ; c'est le PDF qui fait foi pour le reste.
    let png_pl = dossier.join(nom(cle, "couverture", "png"));
    typst.apercu(&src_pl, &png_pl, 1, 72)?;

    Ok(Package {
        cle: cle.to_string(),
        libelle: pr.libelle.clone(),
        papier: papier.nom.clone(),
        pages: interieur.pages,
        gouttiere: interieur.gouttiere,
        blanche: interieur.blanche,
        dos: g.dos,
        dos_requis: planche::dos_insuffisant(livre, cv, g.format.0, g.dos),
        fond_perdu: g.fond_perdu,
        planche: (g.largeur(), g.hauteur()),
        chemins: vec![affiche(&pdf_int), affiche(&pdf_pl)],
        vignette: affiche(&png_pl),
        polices_introuvables,
        interieur_partage,
    })
}

/// Un livrable prêt à packager : sa vue plate, son papier, son relevé et ses clés.
///
/// `cle_gabarit` doit dériver de `pr` ; `lot` mémoïse dessus.
#[derive(Debug, Clone)]
pub struct Cible {
    pub pr: Provider,
    pub papier: Papier,
    pub releve: Releve,
    pub cle: String,
    pub cle_gabarit: String,
}

/// Packager un lot de livrables, l'intérieur composé **une fois par gabarit**.
///
/// Le premier livrable d'un gabarit compose dans son répertoire ; les suivants copient.
/// Un échec de composition ne condamne pas le gabarit : le suivant du même gabarit
/// réessaie, faute d'entrée retenue.
pub fn lot(
    projet: &Projet,
    cibles: &[Cible],
    racine: &Path,
    typst: &Typst,
) -> Vec<Result<Package, String>> {
    let mut prets: BTreeMap<String, (Provider, InterieurCompose)> = BTreeMap::new();
    cibles
        .iter()
        .map(|c| {
            let dossier = racine.join(&c.cle);
            if !prets.contains_key(&c.cle_gabarit) {
                let i = composer_interieur(projet, &c.pr, &c.cle, &dossier, typst)?;
                prets.insert(c.cle_gabarit.clone(), (c.pr.clone(), i));
            }
            let (pr, interieur) = prets
                .get(&c.cle_gabarit)
                .expect("vient d'être inséré si absent");
            debug_assert_eq!(*pr, c.pr, "deux gabarits de même clé, providers différents");
            assembler(
                projet, &c.pr, &c.papier, c.releve, &c.cle, interieur, &dossier, typst,
            )
        })
        .collect()
}

/// Les noms de répertoire des envois, dans l'ordre de la liste.
///
/// Séparé d'`assembler_envois` pour être éprouvé sans toucher au disque ni à Typst :
/// c'est ici que se joue le fait qu'un exemplaire ne parte pas avec le mot d'un autre.
fn dossiers_d_envoi(envois: &[crate::envoi::Envoi]) -> Vec<String> {
    let mut pris: Vec<String> = Vec::with_capacity(envois.len());
    for e in envois {
        let d = crate::envoi::distinct(&crate::envoi::assaini(&e.dedicataire), &pris);
        pris.push(d);
    }
    pris
}

/// Ce qu'un envoi dépose sur sa page, et où il s'y pose : l'image est écrite au passage
/// à côté de la source qui la nommera.
///
/// Écrire l'image ici, et non dans un balayage préalable, garantit qu'aucune image ne
/// se retrouve dans le répertoire d'un autre dédicataire : elle est déposée là où sa
/// source est composée, et elle n'est nommée que par elle.
pub fn trace<'a>(
    projet: &'a Projet,
    e: &'a crate::envoi::Envoi,
    dossier: &Path,
) -> Result<interieur::Trace<'a>, String> {
    let qui = if e.dedicataire.trim().is_empty() {
        "cet envoi"
    } else {
        &e.dedicataire
    };
    let quoi = match &e.main {
        crate::envoi::Main::Police { police } => interieur::Quoi::Texte {
            police,
            texte: &e.contenu,
        },
        // Générée ou écrite à la main, une image est une image : elle a été acceptée,
        // elle est dans l'archive, et composer ne rappelle jamais le réseau.
        crate::envoi::Main::Image | crate::envoi::Main::Diffusion => {
            let fichier = e
                .image
                .as_deref()
                .ok_or_else(|| format!("{qui} n'a pas d'image : en choisir une."))?;
            let octets = projet.images_envois.get(fichier).ok_or_else(|| {
                format!("{qui} : l'image « {fichier} » ne figure pas dans le projet.")
            })?;
            // Détouré ici et nulle part ailleurs : `trace` est le seul chemin par où
            // passent la composition d'un package et le rendu de l'objet du canevas.
            // L'écran ne peut donc pas montrer autre chose que ce qui s'imprime.
            //
            // Le nom passe en `.png` : Typst reconnaît le format d'une image à son
            // extension, et un PNG rangé sous `.jpg` ne se composerait pas.
            let (nom, octets) = match &e.detourage {
                Some(d) => {
                    let png = crate::detourage::applique(octets, d)
                        .map_err(|err| format!("{qui} : {err}"))?;
                    let tige = fichier.rsplit_once('.').map_or(fichier, |(t, _)| t);
                    (format!("{tige}.png"), std::borrow::Cow::Owned(png))
                }
                None => (
                    fichier.to_string(),
                    std::borrow::Cow::Borrowed(octets.as_slice()),
                ),
            };
            std::fs::write(dossier.join(&nom), &*octets)
                .map_err(|err| format!("{nom} : écriture impossible : {err}"))?;
            interieur::Quoi::Image {
                fichier: nom.into(),
            }
        }
    };
    Ok(interieur::Trace {
        quoi,
        place: &e.place,
    })
}

/// Refuse un envoi placé sur une page que l'intérieur de ce prestataire n'a pas.
///
/// Le même manuscrit ne fait pas le même nombre de pages en poche et en grand format.
/// Pour les liminaires — faux-titre, blanche, titre, copyright, dédicace — les pages
/// coïncident d'un format à l'autre, et c'est là qu'un envoi va dans les faits.
/// Ailleurs, on refuse en disant quoi faire, le chiffre mesuré compris : c'est la
/// convention du dos non publié.
fn verifie_pages(liste: &[crate::envoi::Envoi], pages: u32) -> Result<(), String> {
    for (i, e) in liste.iter().enumerate() {
        if e.place.page >= 1 && e.place.page <= pages {
            continue;
        }
        let qui = if e.dedicataire.trim().is_empty() {
            format!("envoi {}", i + 1)
        } else {
            e.dedicataire.clone()
        };
        return Err(format!(
            "{qui} : envoi placé page {}, l'intérieur n'en fait que {pages}.",
            e.place.page
        ));
    }
    Ok(())
}

/// Compose un package par envoi, tous chez le même prestataire.
///
/// **La convergence n'a lieu qu'une fois.** L'envoi se pose par `#place`, qui ne peut
/// pas créer de page : la gouttière, la parité, le compte de pages, le dos et la
/// planche sont donc les mêmes pour tous. Converger M fois ne coûterait pas seulement
/// M fois le temps — cela laisserait croire que le résultat pourrait différer.
pub fn assembler_envois(
    projet: &Projet,
    pr: &Provider,
    papier: &Papier,
    releve: Releve,
    cle: &str,
    racine: &Path,
    typst: &Typst,
) -> Result<Vec<(String, Package)>, String> {
    let envois = &projet.meta.envois;
    envois.verifie()?;
    if envois.liste.is_empty() {
        return Err("aucun envoi : en écrire un avant de générer.".into());
    }

    // Le package de référence, sans envoi : c'est lui qui converge, calcule le dos et
    // compose la planche. Les envois n'en reprennent que le réglage et les fichiers.
    let reference = racine.join(".reference");
    let int = composer_interieur(projet, pr, cle, &reference, typst)?;
    let base = assembler(projet, pr, papier, releve, cle, &int, &reference, typst)?;

    // Le compte de pages n'existe qu'après la convergence : le contrôle ne peut pas
    // avoir lieu plus tôt, et refuser ici coûte une composition de moins qu'un tirage
    // faux.
    verifie_pages(&envois.liste, base.pages)?;

    // La police de l'auteur n'entre en scène qu'ici : le package de référence ne porte
    // aucun envoi, donc aucune écriture manuscrite. Elle est dépliée une fois pour tous
    // les envois, et Typst la cherchera là.
    let typst = &match ecrire_polices(projet, racine)? {
        Some(dossier) => typst.clone().avec_polices(dossier),
        None => typst.clone(),
    };

    let livre = &projet.meta.livre;
    let int_meta = &projet.meta.interieur;
    let chapitres = manuscrit::decoupe(&projet.texte, livre.chapitres)?;
    let reglage = Reglage {
        gouttiere: base.gouttiere,
        blanche: base.blanche,
    };
    let mut sorties = Vec::with_capacity(envois.liste.len());
    for (e, nom_dossier) in envois.liste.iter().zip(dossiers_d_envoi(&envois.liste)) {
        let dossier = racine.join(&nom_dossier);
        std::fs::create_dir_all(&dossier)
            .map_err(|err| format!("répertoire inutilisable ({}) : {err}", dossier.display()))?;

        let src = dossier.join(nom(cle, "interieur", "typ"));
        let t = trace(projet, e, &dossier)?;
        ecrire(
            &src,
            &interieur::source(livre, int_meta, pr, &reglage, &chapitres, Some(t)),
        )?;
        let pdf = dossier.join(nom(cle, "interieur", "pdf"));
        // L'envoi peut composer dans une main que la référence n'emploie pas : ses
        // substitutions à lui s'ajoutent à celles du package de référence.
        let replis = typst.compile(&src, &pdf)?;

        // La planche ne dépend pas de l'envoi : elle est recopiée, pas recomposée.
        let mut p = base.clone();
        for f in replis {
            if !p.polices_introuvables.contains(&f) {
                p.polices_introuvables.push(f);
            }
        }
        p.chemins = vec![
            affiche(&pdf),
            copier(&reference, &dossier, &nom(cle, "couverture", "pdf"))?,
        ];
        p.vignette = copier(&reference, &dossier, &nom(cle, "couverture", "png"))?;
        sorties.push((nom_dossier, p));
    }
    Ok(sorties)
}

/// Recopie un fichier de la référence vers le répertoire d'un envoi, et rend son chemin.
fn copier(depuis: &Path, vers: &Path, fichier: &str) -> Result<String, String> {
    let cible = vers.join(fichier);
    std::fs::copy(depuis.join(fichier), &cible)
        .map_err(|e| format!("{fichier} : copie impossible : {e}"))?;
    Ok(affiche(&cible))
}

/// Quelle face une image sert : c'est son nom qui le dit, et rien d'autre.
///
/// Le projet embarque ses images à plat, sans champ qui leur donnerait un rôle : la
/// convention de nom est donc la seule règle, et elle vaut aussi bien pour l'image
/// importée d'un ancien répertoire de travail que pour celle qu'on choisit dans
/// l'application.
pub fn sert_la_quatrieme(nom: &str) -> bool {
    nom.starts_with("quatrieme")
}

/// Déplie la police personnelle du projet, et rend le répertoire où Typst la trouvera.
///
/// Typst ne lit ses polices que dans des répertoires : l'écriture de l'auteur vit dans
/// le `.ozalid`, elle doit donc atterrir sur le disque avant qu'on puisse composer. Un
/// répertoire à part, et non celui des sorties : `--font-path` est fouillé
/// récursivement, et lui donner le répertoire des envois lui ferait ouvrir un à un tous
/// les PDF qu'on vient d'y écrire.
pub fn ecrire_polices(projet: &Projet, dossier: &Path) -> Result<Option<PathBuf>, String> {
    if projet.polices.is_empty() {
        return Ok(None);
    }
    let cible = dossier.join(".polices");
    std::fs::create_dir_all(&cible)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", cible.display()))?;
    for (nom, octets) in &projet.polices {
        std::fs::write(cible.join(nom), octets).map_err(|e| format!("{nom} : {e}"))?;
    }
    Ok(Some(cible))
}

/// Écrit les images du projet à côté des sources, et rend leurs descriptions.
/// Typst lit ses images par chemin relatif, comme n'importe quel document.
pub fn ecrire_images(
    projet: &Projet,
    dossier: &Path,
) -> Result<(Option<Ressource>, Option<Ressource>), String> {
    let (mut une, mut quatre) = (None, None);
    for (nom, octets) in &projet.images {
        std::fs::write(dossier.join(nom), octets).map_err(|e| format!("{nom} : {e}"))?;
        let r = Ressource::depuis(nom, octets)
            .ok_or_else(|| format!("{nom} : dimensions illisibles (ni PNG ni JPEG)."))?;
        if sert_la_quatrieme(nom) {
            quatre = Some(r);
        } else {
            une = Some(r);
        }
    }
    Ok((une, quatre))
}

fn affiche(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn ecrire(chemin: &Path, contenu: &str) -> Result<(), String> {
    std::fs::write(chemin, contenu)
        .map_err(|e| format!("écriture impossible ({}) : {e}", chemin.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les répertoires d'envoi portent le nom du dédicataire, assaini et rendu unique.
    /// Deux dédicataires qui se confondraient enverraient au second le mot du premier.
    #[test]
    fn les_repertoires_d_envoi_sont_distincts_et_sans_chemin() {
        let envois = [
            crate::envoi::Envoi {
                dedicataire: "Marie/Léa".into(),
                contenu: "A.".into(),
                ..Default::default()
            },
            crate::envoi::Envoi {
                dedicataire: "Marie-Léa".into(),
                contenu: "B.".into(),
                ..Default::default()
            },
            crate::envoi::Envoi {
                dedicataire: "..".into(),
                contenu: "C.".into(),
                ..Default::default()
            },
        ];
        assert_eq!(
            dossiers_d_envoi(&envois),
            vec!["Marie-Léa", "Marie-Léa-2", "envoi"]
        );
    }

    fn projet_en_images(image: Option<&str>) -> Projet {
        let mut p = Projet::nouveau(crate::projet::Livre::vide(), "## 01\n\nA.\n".into());
        p.meta.envois.liste = vec![crate::envoi::Envoi {
            dedicataire: "Léa".into(),
            main: crate::envoi::Main::Image,
            image: image.map(str::to_string),
            ..Default::default()
        }];
        if let Some(n) = image {
            p.images_envois.insert(n.into(), b"\x89PNG".to_vec());
        }
        p
    }

    /// L'image part avec la source qui la nomme, dans le répertoire de son dédicataire :
    /// c'est ce qui garantit qu'aucune image ne se retrouve dans l'exemplaire d'un autre.
    #[test]
    fn l_image_d_un_envoi_est_ecrite_a_cote_de_sa_source() {
        let p = projet_en_images(Some("Léa.png"));
        let dir = tempfile::tempdir().unwrap();
        let t = trace(&p, &p.meta.envois.liste[0], dir.path()).unwrap();

        assert!(matches!(
            &t.quoi,
            interieur::Quoi::Image { fichier } if fichier == "Léa.png"
        ));
        assert_eq!(
            std::fs::read(dir.path().join("Léa.png")).unwrap(),
            b"\x89PNG"
        );
    }

    /// **La promesse du figeage.** Une image générée puis acceptée est une image comme
    /// une autre : elle vit dans l'archive, et composer ne rappelle jamais le modèle. Un
    /// package se refait des mois plus tard, hors ligne, à l'identique — et le jour où
    /// le service aura fermé.
    #[test]
    fn une_image_generee_et_acceptee_compose_comme_une_autre() {
        let mut p = projet_en_images(Some("Léa.png"));
        p.meta.envois.gabarit = "une aquarelle, mention « {envoi} »".into();
        p.meta.envois.liste[0].main = crate::envoi::Main::Diffusion;
        let dir = tempfile::tempdir().unwrap();
        let t = trace(&p, &p.meta.envois.liste[0], dir.path()).unwrap();
        assert!(matches!(
            &t.quoi,
            interieur::Quoi::Image { fichier } if fichier == "Léa.png"
        ));
    }

    /// Un envoi sans image ne compose pas, et l'erreur nomme la personne : la liste peut
    /// en porter dix, et « il manque une image » n'aiderait pas à savoir laquelle. Ce
    /// refus est ici, à la composition, et non à la saisie — on écrit la liste avant de
    /// choisir les images.
    #[test]
    fn un_envoi_sans_image_refuse_de_composer_en_nommant_le_dedicataire() {
        let p = projet_en_images(None);
        let dir = tempfile::tempdir().unwrap();
        let err = trace(&p, &p.meta.envois.liste[0], dir.path()).unwrap_err();
        assert!(err.contains("Léa"), "{err}");
    }

    /// Les deux sorties d'un package portent la clé du livrable entière, cinq segments
    /// compris : deux packages ouverts côte à côte ne peuvent pas être confondus, deux
    /// papiers non plus.
    #[test]
    fn les_sorties_portent_la_cle_du_livrable() {
        assert_eq!(
            nom("bod-135x215-broche-creme-90", "couverture", "pdf"),
            "couverture-bod-135x215-broche-creme-90.pdf"
        );
        assert_eq!(
            nom("bod-135x215-broche-creme-90", "interieur", "typ"),
            "interieur-bod-135x215-broche-creme-90.typ"
        );
    }

    /// `Provider` synthétique, sans dépendance au catalogue : les bornes des vrais POD
    /// refusent un manuscrit d'une page.
    fn provider_d_essai() -> Provider {
        Provider {
            cle: "essai-livre-broche".into(),
            libelle: "Essai — livre".into(),
            format: (135.0, 215.0),
            marge_haut: 18.8,
            marge_bas: 28.0,
            exterieur: 15.0,
            gouttieres: vec![(1, 900, 20.0)],
            fond_perdu: Some(5.0),
            pages_min: 1,
            pages_max: 900,
            papiers: vec![Papier {
                cle: "creme".into(),
                nom: "Crème d'essai".into(),
                teinte: r##"#f7f0e0"##.into(),
                dos: crate::catalogue::Dos::Multiplie {
                    par: 0.0675,
                    plus: 0.6,
                },
                pages: None,
                source: None,
            }],
            fabrication: crate::catalogue::Fabrication {
                pod: "essai".into(),
                format: "livre".into(),
                reliure: "broche".into(),
                papier: "creme".into(),
            },
        }
    }

    fn papier_plafonne(cle: &str, bornes: Option<(u32, u32)>) -> Papier {
        Papier {
            cle: cle.into(),
            nom: format!("papier {cle}"),
            teinte: r##"#ffffff"##.into(),
            dos: crate::catalogue::Dos::Multiplie {
                par: 0.0675,
                plus: 0.6,
            },
            pages: bornes.map(|(min, max)| crate::catalogue::Pagination { min, max }),
            source: None,
        }
    }

    /// Le `Provider` d'essai, sous d'autres bornes de pagination : celui de base en pose
    /// de très larges (1 à 900), et c'est le resserrement qui s'observe ici.
    fn provider_pagine(min: u32, max: u32) -> Provider {
        Provider {
            pages_min: min,
            pages_max: max,
            ..provider_d_essai()
        }
    }

    /// Le plafond d'un papier resserre celui de la reliure, et le refus le nomme.
    ///
    /// BoD accepte 900 pages en broché, mais 868 seulement en photo brillant 130 g : le
    /// plafond appartient au papier, pas à la reliure. Sans ce resserrement, l'application
    /// laisserait composer une couverture pour un livre que l'imprimeur refusera à la
    /// commande — et le dos, lui, serait juste, ce qui rend l'erreur invisible à l'écran.
    #[test]
    fn le_plafond_du_papier_resserre_celui_de_la_reliure() {
        let pr = provider_pagine(24, 900);
        let brillant = papier_plafonne("photo-brillant-130", Some((24, 868)));
        let creme = papier_plafonne("creme-90", None);

        let err = verifie_pagination("bod-135x215-broche-photo-brillant-130", 880, &pr, &brillant)
            .unwrap_err();
        assert!(err.contains("880"), "{err}");
        assert!(err.contains("868"), "{err}");
        assert!(
            err.contains(&brillant.nom),
            "le refus doit nommer le papier qui plafonne : {err}"
        );

        assert!(
            verifie_pagination("bod-135x215-broche-creme-90", 880, &pr, &creme).is_ok(),
            "le même compte passe sur un papier qui ne plafonne pas"
        );
    }

    /// Le refus nomme le papier quand la reliure seule aurait accepté ce compte de pages
    /// — c'est-à-dire quand changer de papier sauverait le livre. Un papier qui ne
    /// resserre que le plancher n'est donc pour rien dans un plafond dépassé : le compte
    /// dépasse ce que la reliure admet, et lui seul.
    #[test]
    fn le_papier_est_nomme_quand_la_reliure_seule_aurait_accepte() {
        let pr = provider_pagine(24, 900);
        let plancher = papier_plafonne("papier-a-plancher", Some((30, 2000)));

        // Le plafond de la reliure est franchi : le papier n'y est pour rien.
        let trop = verifie_pagination("essai", 950, &pr, &plancher).unwrap_err();
        assert!(
            !trop.contains(&plancher.nom),
            "le papier ne resserre pas ce plafond, il ne doit pas être nommé : {trop}"
        );

        // Le plancher, lui, est bien celui du papier : il se nomme.
        let trop_peu = verifie_pagination("essai", 26, &pr, &plancher).unwrap_err();
        assert!(
            trop_peu.contains(&plancher.nom),
            "le plancher franchi est celui du papier : {trop_peu}"
        );
        // Et la borne affichée est celle, resserrée, du papier — 30, pas les 24 de la
        // reliure seule : nommer le papier en montrant le chiffre de la reliure
        // désignerait le bon coupable avec la mauvaise pièce à conviction.
        assert!(
            trop_peu.contains("hors des 30"),
            "la borne affichée doit être celle du papier : {trop_peu}"
        );
    }

    /// Un papier qui resserre le plafond n'est pour rien quand la reliure seule, sans
    /// lui, refuserait déjà ce compte de pages.
    ///
    /// Cas distinct de celui qui précède : ici la borne **affichée** est bien celle du
    /// papier (868 < 900), mais changer de papier ne sauverait pas ce livre — la reliure
    /// seule plafonne à 900, que 950 dépasse tout autant. Nommer le papier enverrait
    /// rouvrir un choix qui n'aurait rien changé.
    #[test]
    fn le_papier_n_est_pas_nomme_si_la_reliure_seule_refuserait_deja() {
        let pr = provider_pagine(24, 900);
        let brillant = papier_plafonne("photo-brillant-130", Some((24, 868)));

        let err = verifie_pagination("essai", 950, &pr, &brillant).unwrap_err();
        assert!(
            !err.contains(&brillant.nom),
            "la reliure seule refuserait déjà 950 pages, le papier ne doit pas être nommé : {err}"
        );
    }

    /// Le `Livre` du témoin, réduit à ce qu'`assembler` et `composer_interieur` lisent.
    fn livre_d_essai() -> crate::projet::Livre {
        crate::projet::Livre {
            titre: "Essai".into(),
            titre_page: "%TITRE%".into(),
            auteur: "Autrice".into(),
            genre: "essai".into(),
            editeur: "Editeur".into(),
            collection: "Collection".into(),
            monogramme: "M".into(),
            copyright: "Domaine public.".into(),
            prix: "Prix".into(),
            mention: "Mention".into(),
            dedicace: String::new(),
            chapitres: Some(1),
        }
    }

    /// Un intérieur déjà composé n'est pas recomposé : `assembler` reçoit l'intérieur
    /// d'un gabarit et ne rappelle Typst que pour la planche. Preuve par l'ordre des
    /// refus : avec un binaire Typst inexistant **et** un intérieur prêt, l'échec doit
    /// venir de la maquette absente (étape planche) — si l'intérieur était recomposé, il
    /// viendrait de Typst, avant elle.
    #[test]
    fn un_interieur_pret_n_est_pas_recompose() {
        let projet = Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        // Pas de maquette de couverture : c'est le refus attendu.
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("interieur-essai.typ");
        let pdf = dir.path().join("interieur-essai.pdf");
        std::fs::write(&src, "source factice").unwrap();
        std::fs::write(&pdf, "pdf factice").unwrap();
        let pret = InterieurCompose {
            pages: 100,
            gouttiere: 20.0,
            blanche: false,
            polices_introuvables: vec![],
            src,
            pdf,
        };
        let pr = provider_d_essai();
        let e = assembler(
            &projet,
            &pr,
            &pr.papiers[0],
            Releve::default(),
            "essai",
            &pret,
            dir.path(),
            &Typst::new("typst-qui-n-existe-pas"),
        )
        .unwrap_err();
        assert!(e.contains("maquette"), "l'intérieur a été recomposé : {e}");
    }

    /// Spec § 9 : deux livrables du même gabarit d'intérieur ne déclenchent **qu'une**
    /// composition. Composition réelle (Typst du PATH, comme `interieur.rs` le fait
    /// déjà) : le second package porte un intérieur copié, pas recomposé, et les deux
    /// PDF sont identiques à l'octet.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn deux_livrables_du_meme_gabarit_ne_composent_l_interieur_qu_une_fois() {
        let mut projet = Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        projet.meta.couverture.maquette = Some(
            crate::maquettes::par_cle(None, "filets")
                .expect("maquette fournie « filets »")
                .couverture,
        );
        let racine = tempfile::tempdir().unwrap();
        let typst = Typst::new("typst")
            .avec_polices(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"));
        let pr = provider_d_essai();
        let creme = pr.papiers[0].clone();
        let blanc = Papier {
            cle: "blanc-essai".into(),
            nom: "Blanc d'essai".into(),
            teinte: r##"#ffffff"##.into(),
            // `plus` largement au-dessus de celui du crème : la formule doit rendre un
            // dos plus épais quel que soit le nombre de pages, court comme long.
            dos: crate::catalogue::Dos::Multiplie {
                par: 0.08,
                plus: 2.0,
            },
            pages: None,
            source: None,
        };
        let cibles = [
            Cible {
                pr: pr.clone(),
                papier: creme,
                releve: Releve::default(),
                cle: "essai-livre-broche-creme".into(),
                cle_gabarit: "essai-livre-broche".into(),
            },
            Cible {
                pr,
                papier: blanc,
                releve: Releve::default(),
                cle: "essai-livre-broche-blanc-essai".into(),
                cle_gabarit: "essai-livre-broche".into(),
            },
        ];
        let sorties = lot(&projet, &cibles, racine.path(), &typst);
        let [a, b]: [&Package; 2] = [
            sorties[0].as_ref().expect("premier package"),
            sorties[1].as_ref().expect("second package"),
        ];
        assert!(!a.interieur_partage, "le premier compose");
        assert!(b.interieur_partage, "le second copie");
        assert_eq!(a.pages, b.pages);
        assert!(b.dos > a.dos, "le papier plus épais fait un dos plus épais");
        let lu = |cle: &str| {
            std::fs::read(racine.path().join(cle).join(format!("interieur-{cle}.pdf"))).unwrap()
        };
        assert_eq!(
            lu("essai-livre-broche-creme"),
            lu("essai-livre-broche-blanc-essai"),
            "le même intérieur, à l'octet"
        );
    }

    /// `lot` rend un résultat par cible, dans l'ordre, même quand tout échoue : c'est ce
    /// contrat-là que `commands::packager` consomme par `paquets.next().expect(…)`.
    /// Et un gabarit dont la composition a échoué n'est pas retenu — la cible suivante
    /// retente au lieu d'hériter d'un échec.
    #[test]
    fn lot_rend_un_resultat_par_cible_et_ne_condamne_pas_le_gabarit() {
        let projet = Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        let racine = tempfile::tempdir().unwrap();
        let pr = provider_d_essai();
        let c = |cle: &str| Cible {
            pr: pr.clone(),
            papier: pr.papiers[0].clone(),
            releve: Releve::default(),
            cle: cle.into(),
            cle_gabarit: "essai-livre-broche".into(),
        };
        let sorties = lot(
            &projet,
            &[c("essai-a"), c("essai-b")],
            racine.path(),
            &Typst::new("typst-absent"),
        );
        assert_eq!(sorties.len(), 2, "un résultat par cible");
        for (i, s) in sorties.iter().enumerate() {
            let e = s
                .as_ref()
                .err()
                .unwrap_or_else(|| panic!("cible {i} aurait dû échouer"));
            assert!(e.contains("typst-absent"), "cible {i} : {e}");
        }
    }

    /// Le même manuscrit ne fait pas le même nombre de pages en poche et en grand
    /// format : une page choisie à l'œil chez l'un peut n'exister chez l'autre. Rogner
    /// sur la dernière page enverrait à l'impression un exemplaire que personne n'a
    /// voulu ; le refus nomme la personne, la page et le compte, comme le fait déjà le
    /// dos non publié.
    #[test]
    fn une_page_hors_bornes_fait_refuser_la_generation() {
        let err = verifie_pages(
            &[
                crate::envoi::Envoi {
                    dedicataire: "Léa".into(),
                    place: crate::envoi::Place {
                        page: 3,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                crate::envoi::Envoi {
                    dedicataire: "Marc".into(),
                    place: crate::envoi::Place {
                        page: 210,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            198,
        )
        .unwrap_err();
        assert!(err.contains("Marc"), "{err}");
        assert!(err.contains("210"), "{err}");
        assert!(err.contains("198"), "{err}");
        assert!(!err.contains("Léa"), "Léa n'est pas en cause : {err}");
    }

    /// Page 0 n'existe pas : les pages de Typst comptent à partir de 1, et un zéro
    /// venu d'un TOML écrit à la main ne doit pas composer un envoi invisible.
    #[test]
    fn la_page_zero_est_refusee() {
        let err = verifie_pages(
            &[crate::envoi::Envoi {
                dedicataire: "Léa".into(),
                place: crate::envoi::Place {
                    page: 0,
                    ..Default::default()
                },
                ..Default::default()
            }],
            198,
        )
        .unwrap_err();
        assert!(err.contains("Léa"), "{err}");
    }

    /// Ce que `trace` écrit sur le disque est détouré, et porte un nom en `.png` : Typst
    /// reconnaît le format d'une image **à son extension**, et un PNG rangé sous `.jpg`
    /// ne se composerait pas — l'erreur tomberait sur l'exemplaire d'une personne.
    #[test]
    fn une_image_detouree_s_ecrit_en_png() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = projet_en_images(Some("Léa.jpg"));
        // Un JPEG uni clair : tout est papier, donc tout doit sortir transparent.
        let mut jpeg = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            8,
            image::Rgb([245, 243, 238]),
        ))
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
        p.images_envois.insert("Léa.jpg".into(), jpeg);
        p.meta.envois.liste[0].detourage = Some(crate::detourage::Detourage {
            papier: 240.0,
            encre: 40.0,
        });

        let t = trace(&p, &p.meta.envois.liste[0], dir.path()).unwrap();
        let interieur::Quoi::Image { fichier } = t.quoi else {
            panic!("la trace n'est pas une image");
        };
        assert!(fichier.ends_with(".png"), "écrit sous « {fichier} »");
        let ecrit = std::fs::read(dir.path().join(&*fichier)).unwrap();
        let px = image::load_from_memory(&ecrit).unwrap().to_rgba8();
        assert_eq!(
            px.get_pixel(0, 0)[3],
            0,
            "le papier n'a pas été rendu transparent"
        );
    }

    /// Un projet d'avant ce chantier compose exactement ce qu'il composait : mêmes
    /// octets, même nom. C'est l'autre moitié de la décision « un projet ancien garde
    /// son rendu » — la première moitié est dans `envoi.rs`, et elle ne dit que le
    /// modèle.
    #[test]
    fn sans_detourage_l_image_part_telle_quelle() {
        let dir = tempfile::tempdir().unwrap();
        let mut p = projet_en_images(Some("Léa.jpg"));
        let mut jpeg = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            8,
            8,
            image::Rgb([245, 243, 238]),
        ))
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            image::ImageFormat::Jpeg,
        )
        .unwrap();
        p.images_envois.insert("Léa.jpg".into(), jpeg.clone());
        // Le projet ancien : la photo est là, les seuils n'y sont pas.
        p.meta.envois.liste[0].detourage = None;

        let t = trace(&p, &p.meta.envois.liste[0], dir.path()).unwrap();
        let interieur::Quoi::Image { fichier } = t.quoi else {
            panic!("la trace n'est pas une image");
        };
        assert!(fichier.ends_with(".jpg"), "le nom a changé : « {fichier} »");
        assert_eq!(
            std::fs::read(dir.path().join(&*fichier)).unwrap(),
            jpeg,
            "les octets ont été retouchés sans qu'on l'ait demandé"
        );
    }
}
