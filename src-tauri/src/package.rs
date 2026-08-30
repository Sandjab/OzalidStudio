//! Le package d'un livrable : l'intérieur, la planche, et de quoi les relire.
//!
//! Un livre, N livrables, aucun réglage retouché entre les deux — c'est la « file
//! d'attente » du COOKBOOK, exécutée. Chaque livrable déclenche sa propre
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
    /// tient, pour ce livrable-là, avec le dos qui a réellement été mesuré.
    pub vignette: String,
    /// Familles que Typst n'a pas trouvées et a remplacées par une écriture de repli
    /// — sans échouer, donc sans que rien d'autre ne le dise. Vide, tout va bien.
    pub polices_introuvables: Vec<String>,
    /// Ce que la composition a relevé sans échouer : une image trop pauvre pour
    /// l'impression, un texte au dos sous le seuil de l'imprimeur. Des phrases toutes
    /// faites — voir [`avertissements`] —, que le compte rendu affiche telles quelles.
    pub avertissements: Vec<String>,
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
    papier: &Papier,
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
            &interieur::source(livre, int, pr, &papier.nom, reglage, &chapitres, None),
        )?;
        typst.pages(&src)
    })?;
    let reglage = Reglage {
        gouttiere: r.gouttiere,
        blanche: r.blanche,
    };
    ecrire(
        &src,
        &interieur::source(livre, int, pr, &papier.nom, &reglage, &chapitres, None),
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

/// La colonne où commence la valeur d'une ligne de fiche.
///
/// Dix-sept caractères : « Fond perdu » et « Gouttière », les deux plus longs libellés,
/// y tiennent avec un blanc de respiration. Une fiche se lit à côté d'un formulaire
/// qu'on remplit — la colonne est ce qui permet de descendre les yeux au lieu de
/// chercher chaque valeur.
const COLONNE: usize = 17;

/// Un nombre tel que la fenêtre l'écrit : virgule décimale, nombre de décimales choisi.
///
/// Deux écritures d'un même millimètre — « 16,51 » à l'écran, « 16.51 » sur la fiche —
/// se lisent comme deux mesures. C'est la même règle que `nb` dans `app.js`.
fn nb(v: f64, d: usize) -> String {
    format!("{v:.d$}").replace('.', ",")
}

/// Une ligne de fiche, ou rien quand la valeur est vide.
///
/// Une entrée vide sur un document qu'on lit devant le formulaire d'un imprimeur se
/// lirait comme un réglage manqué : la finition suit à l'écran la même règle.
fn ligne(cle: &str, valeur: &str) -> String {
    if valeur.is_empty() {
        return String::new();
    }
    format!("{cle:<COLONNE$}{valeur}\n")
}

/// Le nom d'un fichier, sans le répertoire qui le porte : la fiche vit à côté de lui.
fn nom_seul(chemin: &str) -> &str {
    chemin.rsplit(['/', '\\']).next().unwrap_or(chemin)
}

/// La fiche de téléversement d'un livrable : ce qu'il faut saisir chez l'imprimeur, et
/// ce que la composition a mesuré pour ce livrable-là.
///
/// **Entièrement tirée du catalogue et des mesures, jamais du COOKBOOK** : celui-ci est
/// de la prose relue par un humain, et en dupliquer les tableaux en données créerait
/// deux vérités qui divergeraient au premier changement de guide.
///
/// Elle existe parce que le dossier livré ne porte que des PDF muets : ni le format
/// commandé, ni le papier, ni le dos qu'ils supposent. Tout cela vivait à l'écran, et le
/// formulaire de l'imprimeur se remplissait de mémoire, des semaines plus tard.
///
/// Les avertissements viennent tels quels du package : ils sont écrits une fois, en
/// Rust, précisément pour que la fiche et la fenêtre disent la même chose mot pour mot.
pub fn fiche(
    livre: &crate::projet::Livre,
    pr: &Provider,
    finition: Option<&str>,
    p: &Package,
) -> String {
    let (titre, auteur) = (livre.titre.trim(), livre.auteur.trim());
    let entete = match (titre, auteur) {
        ("", "") => String::new(),
        (t, "") => t.to_owned(),
        ("", a) => a.to_owned(),
        (t, a) => format!("{t} — {a}"),
    };

    let mut s = String::new();
    if !entete.is_empty() {
        s.push_str(&format!("{entete}\n\n"));
    }
    s.push_str(&ligne("Imprimeur", &pr.pod_nom));
    s.push_str(&ligne("Format", &pr.format_nom));
    s.push_str(&ligne("Reliure", &pr.reliure_nom));
    s.push_str(&ligne("Papier", &p.papier));
    s.push_str(&ligne("Finition", finition.unwrap_or("")));

    // La blanche de parité se dit là où il y en a une : c'est une page que l'auteur n'a
    // pas écrite et qui se compte quand même, et elle explique un total impair devenu
    // pair sous les yeux de qui relit la commande.
    let pages = if p.blanche {
        format!("{} (dont une blanche de parité)", p.pages)
    } else {
        p.pages.to_string()
    };
    s.push_str(&format!("\n{}", ligne("Pages", &pages)));
    s.push_str(&ligne("Dos", &format!("{} mm", nb(p.dos, 2))));
    s.push_str(&ligne("Gouttière", &format!("{} mm", nb(p.gouttiere, 1))));
    s.push_str(&ligne("Fond perdu", &format!("{} mm", nb(p.fond_perdu, 3))));
    s.push_str(&ligne(
        "Planche",
        &format!("{} × {} mm", nb(p.planche.0, 2), nb(p.planche.1, 2)),
    ));

    // Les fichiers qui partent chez l'imprimeur, et eux seuls : la vignette n'est pas
    // dans `chemins`, et la fiche ne se liste pas elle-même — elle y entre après avoir
    // été écrite.
    s.push('\n');
    for (i, c) in p.chemins.iter().enumerate() {
        let cle = if i == 0 { "Fichiers" } else { "" };
        s.push_str(&format!("{cle:<COLONNE$}{}\n", nom_seul(c)));
    }

    s.push_str(
        "\nLe papier commandé doit être celui déclaré : c'est lui qui porte l'épaisseur \
         du dos.\n",
    );

    // Ce que la composition a relevé sans échouer. La phrase du repli est celle de
    // `livraison.js` : la même alerte, dans le dossier qu'elle a traversé.
    if !p.polices_introuvables.is_empty() {
        s.push_str(&format!(
            "\nPolice introuvable, composé dans une écriture de repli : {}. Le PDF ne \
             suit pas la maquette.\n",
            p.polices_introuvables.join(", ")
        ));
    }
    for a in &p.avertissements {
        s.push_str(&format!("\n{a}\n"));
    }
    s
}

/// La résolution sous laquelle une image imprimée se voit.
///
/// **Convention d'Ozalid, et non un seuil relevé chez un imprimeur** : aucun des six ne
/// publie de minimum. C'est la valeur d'usage de la photogravure, écrite une fois et
/// nommée pour qu'on sache d'où elle vient le jour où on la discutera.
const PPP_MINIMUM: f64 = 300.0;

/// L'avertissement d'une image trop pauvre pour l'impression, s'il y a lieu.
///
/// La mesure est prise sur les millimètres que l'image occupe **une fois cadrée et
/// zoomée**, et non sur la zone : une image recadrée à 40 % n'imprime que 40 % de ses
/// pixels, et la juger sur ses pixels bruts la déclarerait bonne à tort.
///
/// Un avertissement et jamais un refus, contrairement à l'ISBN : une image à 250 ppp
/// s'imprime, et le tirage reste juste. C'est un jugement d'auteur.
fn image_pauvre(fichier: &str, pixels: u32, mm: f64) -> Option<String> {
    if mm <= 0.0 {
        return None;
    }
    let ppp = f64::from(pixels) / mm * 25.4;
    (ppp < PPP_MINIMUM).then(|| {
        format!(
            "Image « {fichier} » posée à {ppp:.0} ppp, sous les {PPP_MINIMUM:.0} ppp \
             d'une impression : elle s'imprimera floue. La recadrer moins, ou en \
             fournir une plus définie."
        )
    })
}

/// L'avertissement d'un texte au dos que l'imprimeur n'autorise pas à cette pagination.
///
/// Deux conditions, et il faut les deux : la pagination est sous le seuil publié, **et**
/// le dos compose au moins un élément. Un dos nu sous le seuil ne pose aucun problème —
/// c'est le texte que le guide refuse sur une tranche mince, pas la tranche.
///
/// Un imprimeur qui ne publie pas de seuil ne contrôle rien : `dos_texte_pages` est
/// `None` chez quatre des six, et son absence vaut silence. Un seuil inventé ferait
/// éteindre un dos que le guide autorise.
///
/// Un avertissement et non un refus : le PDF composé reste juste, et c'est l'imprimeur
/// qui décidera de l'imprimer. Le sien est le seul avis qui compte.
fn dos_sous_le_seuil(pr: &Provider, pages: u32, porte_du_texte: bool) -> Option<String> {
    let seuil = pr.dos_texte_pages?;
    (porte_du_texte && pages < seuil).then(|| {
        format!(
            "Texte au dos sur {pages} pages : {} n'en autorise qu'à partir de {seuil}. \
             Éteindre les éléments du dos.",
            pr.libelle
        )
    })
}

/// L'avertissement de l'image d'un envoi, s'il y a lieu.
///
/// Sa largeur imprimée est une fraction de celle de la page — `place.taille` —, comme
/// `foreground` la compose : c'est le réglage qu'on tire à la souris sur le canevas, et
/// il pèse sur la résolution autant que le recadrage d'une couverture.
///
/// Le détourage ne change pas les pixels, seulement leur transparence : les dimensions
/// de l'archive sont donc celles qui s'impriment, et il n'y a rien à recalculer.
///
/// Rien à dire d'un envoi manuscrit, ni d'une image absente de l'archive ou illisible :
/// le refus de composer, lui, est déjà porté par `trace`.
fn image_d_envoi_pauvre(
    projet: &Projet,
    e: &crate::envoi::Envoi,
    largeur_page: f64,
) -> Option<String> {
    let fichier = e.image.as_deref()?;
    let (pixels, _) = crate::image::dimensions(projet.images_envois.get(fichier)?)?;
    image_pauvre(fichier, pixels, e.place.taille * largeur_page)
}

/// Ce que la composition d'un livrable a relevé **sans échouer** : une image trop pauvre
/// pour l'impression, un texte au dos sous le seuil de l'imprimeur.
///
/// Des phrases toutes faites, et non des mesures à mettre en forme : le compte rendu les
/// affiche telles quelles, et la fiche de téléversement les recopiera — un dossier relu
/// trois mois plus tard doit dire ce que l'écran disait, mot pour mot.
///
/// Hors d'`assembler` pour être éprouvé sans Typst ni disque, comme `verifie_pagination`.
///
/// Une même image posée sur deux faces — c'est le prolongement panoramique — n'avertit
/// qu'une fois : elle est pauvre une fois, pas deux.
pub fn avertissements(
    livre: &crate::projet::Livre,
    cv: &crate::couverture::Couverture,
    pr: &Provider,
    g: &Gabarit,
    pages: u32,
    une: Option<&Ressource>,
    quatre: Option<&Ressource>,
) -> Vec<String> {
    let mut v = Vec::new();
    let mut vues: Vec<&str> = Vec::new();
    for (r, largeur) in planche::images_posees(cv, g, une, quatre) {
        if vues.contains(&r.fichier.as_str()) {
            continue;
        }
        vues.push(&r.fichier);
        v.extend(image_pauvre(&r.fichier, r.largeur, largeur));
    }
    v.extend(dos_sous_le_seuil(
        pr,
        pages,
        planche::dos_porte_du_texte(livre, cv),
    ));
    v
}

/// Assemble un package : le dos, découlant de la pagination de l'intérieur, puis la
/// planche. L'intérieur lui-même n'est pas recomposé — il est reçu tout fait, composé
/// une fois par gabarit par `composer_interieur` (directement ou via `lot`), et copié
/// ici s'il vient d'un autre répertoire que celui de ce livrable.
///
/// Le `releve` ne sert que chez les imprimeurs qui ne publient ni dos ni fond perdu ;
/// ailleurs, il est ignoré au profit de leur formule.
pub fn assembler(
    projet: &Projet,
    c: &Cible,
    interieur: &InterieurCompose,
    dossier: &Path,
    typst: &Typst,
) -> Result<Package, String> {
    let Cible {
        pr,
        papier,
        releve,
        finition,
        cle,
    } = c;
    let (cle, finition) = (cle.as_str(), finition.as_deref());
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
    let g = Gabarit::pour(pr, papier, interieur.pages, *releve)?;

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
        &planche::source(
            &crate::gabarit::Contexte {
                livre,
                marques: crate::gabarit::Marques::chez(pr).sur(&papier.nom),
            },
            cv,
            &g,
            une.as_ref(),
            quatre.as_ref(),
        )?,
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

    let mut p = Package {
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
        avertissements: avertissements(
            livre,
            cv,
            pr,
            &g,
            interieur.pages,
            une.as_ref(),
            quatre.as_ref(),
        ),
        interieur_partage,
    };

    // 5. La fiche, écrite en dernier : elle recopie ce que le package vient de mesurer,
    // avertissements compris, et elle n'entre dans ses chemins qu'après — une fiche qui
    // se listerait elle-même parmi les fichiers à téléverser enverrait déposer un
    // mémo chez l'imprimeur.
    p.chemins
        .push(ecrire_fiche(livre, pr, finition, &p, dossier)?);
    Ok(p)
}

/// Écrit la fiche de téléversement à côté des PDF, et rend son chemin.
///
/// Le nom ne porte pas la clé du livrable, contrairement aux PDF : il n'y a qu'une fiche
/// par répertoire, et c'est elle qui dit de quel livrable il s'agit — dès sa première
/// ligne.
fn ecrire_fiche(
    livre: &crate::projet::Livre,
    pr: &Provider,
    finition: Option<&str>,
    p: &Package,
    dossier: &Path,
) -> Result<String, String> {
    let chemin = dossier.join("televersement.txt");
    ecrire(&chemin, &fiche(livre, pr, finition, p))?;
    Ok(affiche(&chemin))
}

/// Un livrable prêt à packager : sa vue plate, son papier, son relevé, sa finition et sa
/// clé.
///
/// **C'est l'argument de `assembler` et d'`assembler_envois`**, et non cinq paramètres
/// côte à côte : les cinq vont ensemble, ils viennent tous du même livrable, et les
/// passer séparément laissait à chaque appelant l'occasion de les dépareiller — un
/// papier d'un livrable avec la clé d'un autre écrit un dos faux dans le bon répertoire.
///
/// [`composer_interieur`] ne la prend pas, et c'est délibéré : la pagination ne dépend
/// ni du papier ni de la finition, et lui donner la cible entière laisserait croire le
/// contraire. Sa signature dit ce qu'elle lit — le gabarit, et la clé qui nomme ses
/// fichiers.
///
/// La clé du **gabarit**, sur laquelle `lot` mémoïse, n'y figure pas : c'est `pr.cle`,
/// que ses deux constructeurs posent à `fabrication.cle_gabarit()`. La porter en double
/// obligeait les appelants qui ne mémoïsent pas à la fournir quand même, et faisait d'un
/// fait de construction un invariant à tenir.
#[derive(Debug, Clone)]
pub struct Cible {
    pub pr: Provider,
    pub papier: Papier,
    pub releve: Releve,
    /// La finition du livrable sous son **nom d'imprimeur**, quand il en porte une :
    /// c'est « Pelliculage mat » qu'on coche sur un bon de commande, pas « mat ». Elle
    /// ne change pas un octet du PDF — c'est une donnée de commande —, et c'est à ce
    /// titre qu'elle entre sur la fiche de téléversement.
    pub finition: Option<String>,
    /// La clé du **livrable**, à quatre axes : celle qui nomme son répertoire et ses
    /// fichiers.
    pub cle: String,
}

impl Cible {
    /// La fabrication de ce livrable : le triplet du provider, et **son** papier.
    ///
    /// `pr.fabrication` ne convient pas et ne le peut pas : son papier est celui par
    /// défaut du POD — un `Provider` vaut pour un triplet, plusieurs papiers s'y
    /// rattachent. La lui prendre, c'est ranger l'intérieur d'un papier sous la clé d'un
    /// autre, exactement le défaut que ce type existe pour éviter (`commands::cible`).
    fn fabrication(&self) -> crate::catalogue::Fabrication {
        crate::catalogue::Fabrication {
            papier: self.papier.cle.clone(),
            ..self.pr.fabrication.clone()
        }
    }
}

/// Les deux `Provider` décrivent-ils le même gabarit d'intérieur ?
///
/// Séparé de `lot` pour être éprouvé sans Typst, comme `dossiers_d_envoi` l'est du
/// disque : c'est ici que se joue le fait qu'un intérieur mémoïsé ne soit pas resservi à
/// un gabarit qui n'est pas le sien.
///
/// La `fabrication` est le seul champ qui varie légitimement entre deux `Provider` de
/// même clé, et c'est par elle que la comparaison entière échouait : `Resolu::provider`
/// pose la clé sur le triplet mais garde la fabrication du **livrable**, papier compris —
/// or un intérieur pour N papiers est précisément ce que `lot` mémoïse. La neutraliser
/// laisse porter la comparaison sur tout le reste du `Provider`, champ neuf compris, sans
/// qu'il faille les énumérer ici ; ses trois autres segments, eux, sont déjà dans `cle`.
fn meme_gabarit(a: &Provider, b: &Provider) -> bool {
    *a == Provider {
        fabrication: a.fabrication.clone(),
        ..b.clone()
    }
}

/// L'intérieur qu'un autre livrable du même gabarit a déjà sur le disque, prêt à être copié.
///
/// C'est ce qui fait que trois papiers d'un même gabarit coûtent une composition et deux
/// copies **qu'on les ajoute d'un coup ou un par un** (spec § 4). La pagination vient de la
/// mesure retenue, jamais du PDF, qui ne la porte pas — et une mesure absente refuse donc le
/// prêt, ce qui ne peut pas être un faux négatif : les trois mutateurs qui l'effacent changent
/// tous l'empreinte d'intérieur (reconnaissance 4c).
///
/// **Les cibles de la passe sont exclues.** Sans quoi une régénération se prêterait son propre
/// intérieur : le garde-fou d'`assembler` éviterait la troncature, mais rien ne serait
/// recomposé et le package s'annoncerait pourtant partagé. C'est aussi ce qui laisse « Tout
/// regénérer », dont toutes les cibles sont dans la passe, composer exactement comme avant.
fn interieur_du_disque(
    projet: &Projet,
    racine: &Path,
    fabrication: &crate::catalogue::Fabrication,
    exclues: &[&str],
) -> Option<InterieurCompose> {
    let m = projet.meta.livraison.mesure(&fabrication.cle_gabarit())?;
    projet.meta.livraison.livrables.iter().find_map(|l| {
        // La fabrication entière et non le seul gabarit : depuis `%PAPIER%`, deux papiers
        // du même gabarit ne composent plus le même intérieur, et le prêt écrirait le nom
        // d'un papier dans le livrable de l'autre. La mesure, elle, reste rangée par
        // gabarit — c'est la pagination, et le papier ne la change pas.
        if l.fabrication.cle() != fabrication.cle() {
            return None;
        }
        let cle = l.cle();
        if exclues.contains(&cle.as_str()) {
            return None;
        }
        let crate::projet::Generation::Fait { interieur, .. } = &l.generation else {
            return None;
        };
        if *interieur != crate::empreinte::interieur(projet, l) {
            return None;
        }
        // Les deux fichiers, jamais un seul : `assembler` copie la source avec le PDF, et un
        // `.typ` manquant laisserait dans le répertoire livré une source qui ne correspond à
        // rien.
        let dossier = racine.join(&cle);
        let src = dossier.join(nom(&cle, "interieur", "typ"));
        let pdf = dossier.join(nom(&cle, "interieur", "pdf"));
        (src.is_file() && pdf.is_file()).then(|| InterieurCompose {
            pages: m.pages,
            gouttiere: m.gouttiere,
            blanche: m.blanche,
            polices_introuvables: m.polices_introuvables.clone(),
            src,
            pdf,
        })
    })
}

/// Packager un lot de livrables, l'intérieur composé **une fois par gabarit et par papier**.
///
/// Le premier livrable d'une fabrication compose dans son répertoire ; les suivants
/// copient. Un échec de composition ne la condamne pas : le suivant réessaie, faute
/// d'entrée retenue.
///
/// Le papier est entré dans la clé avec `%PAPIER%` : deux papiers du même gabarit
/// partagent une pagination — c'est l'invariant de la mesure — mais plus une source, dès
/// qu'un champ libre nomme le papier. Ce que le lot cesse de partager, il ne le partageait
/// qu'entre papiers d'un même gabarit : le reste est intact.
pub fn lot(
    projet: &Projet,
    cibles: &[Cible],
    racine: &Path,
    typst: &Typst,
) -> Vec<Result<Package, String>> {
    let mut prets: BTreeMap<String, (Provider, InterieurCompose)> = BTreeMap::new();
    // Les cibles de cette passe ne se prêtent rien : elles sont là pour être composées.
    let exclues: Vec<&str> = cibles.iter().map(|c| c.cle.as_str()).collect();
    cibles
        .iter()
        .map(|c| {
            let dossier = racine.join(&c.cle);
            let partage = c.fabrication().cle();
            if !prets.contains_key(&partage) {
                let i = match interieur_du_disque(projet, racine, &c.fabrication(), &exclues) {
                    Some(i) => i,
                    None => composer_interieur(projet, &c.pr, &c.papier, &c.cle, &dossier, typst)?,
                };
                prets.insert(partage.clone(), (c.pr.clone(), i));
            }
            let (pr, interieur) = prets.get(&partage).expect("vient d'être inséré si absent");
            debug_assert!(
                meme_gabarit(pr, &c.pr),
                "deux gabarits de même clé, providers différents"
            );
            assembler(projet, c, interieur, &dossier, typst)
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

/// Refuse un envoi placé sur une page que l'intérieur de ce livrable n'a pas.
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

/// Compose un package par envoi, tous pour le même livrable.
///
/// **La convergence n'a lieu qu'une fois.** L'envoi se pose par `#place`, qui ne peut
/// pas créer de page : la gouttière, la parité, le compte de pages, le dos et la
/// planche sont donc les mêmes pour tous. Converger M fois ne coûterait pas seulement
/// M fois le temps — cela laisserait croire que le résultat pourrait différer.
pub fn assembler_envois(
    projet: &Projet,
    c: &Cible,
    racine: &Path,
    typst: &Typst,
) -> Result<Vec<(String, Package)>, String> {
    let (pr, papier, cle, finition) = (&c.pr, &c.papier, c.cle.as_str(), c.finition.as_deref());
    let envois = &projet.meta.envois;
    envois.verifie()?;
    if envois.liste.is_empty() {
        return Err("aucun envoi : en écrire un avant de générer.".into());
    }

    // Le package de référence, sans envoi : c'est lui qui converge, calcule le dos et
    // compose la planche. Les envois n'en reprennent que le réglage et les fichiers.
    let reference = racine.join(".reference");
    let int = composer_interieur(projet, pr, papier, cle, &reference, typst)?;
    let base = assembler(projet, c, &int, &reference, typst)?;

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
            &interieur::source(
                livre,
                int_meta,
                pr,
                &papier.nom,
                &reglage,
                &chapitres,
                Some(t),
            ),
        )?;
        let pdf = dossier.join(nom(cle, "interieur", "pdf"));
        // L'envoi peut composer dans une main que la référence n'emploie pas : ses
        // substitutions à lui s'ajoutent à celles du package de référence.
        let replis = typst.compile(&src, &pdf)?;

        // La planche ne dépend pas de l'envoi : elle est recopiée, pas recomposée.
        let mut p = base.clone();
        // La sienne, en revanche, lui appartient : la même main posée deux fois plus
        // grande n'a pas la même définition, et c'est un réglage par exemplaire.
        p.avertissements
            .extend(image_d_envoi_pauvre(projet, e, pr.format.0));
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
        p.chemins
            .push(ecrire_fiche(livre, pr, finition, &p, &dossier)?);
        // Réécrite et non recopiée : celle de la référence ne porte ni l'avertissement
        // de cet exemplaire-ci ni sa police de repli, et un dossier livré doit dire ce
        // qu'il contient, lui et pas un autre.

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
    ecrire_table(&projet.images, dossier)
}

/// La même chose, pour une table d'images qui n'est celle d'aucun projet : la vignette
/// d'une maquette compose les photos du livre et celles de l'archive fondues, et rien
/// dans ce montage n'est un `Projet`.
pub fn ecrire_table(
    images: &BTreeMap<String, Vec<u8>>,
    dossier: &Path,
) -> Result<(Option<Ressource>, Option<Ressource>), String> {
    let (mut une, mut quatre) = (None, None);
    for (nom, octets) in images {
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

/// Ce qu'une suppression a laissé derrière elle.
#[derive(Debug, Default, Serialize)]
pub struct Nettoyage {
    /// Fichiers connus qui n'étaient plus là. Ce n'est pas une erreur — une génération échouée
    /// n'en écrit qu'une partie —, mais le compte rendu le dit.
    pub absents: Vec<String>,
    /// Ce qui restait et que l'application n'a pas écrit. Le répertoire survit pour eux.
    pub etrangers: Vec<String>,
    /// Le répertoire lui-même est parti : il ne restait rien.
    pub dossier_retire: bool,
}

/// Les fichiers que l'application écrit dans le répertoire d'un livrable.
///
/// Cinq noms se reconstruisent de la clé (`nom`), la fiche n'en porte pas — il n'y en a qu'une
/// par répertoire —, et les images de couverture sont écrites sous **leur nom d'origine** par
/// `ecrire_table`. Cette dernière liste est celle du projet **courant** : une image retirée du
/// projet après la génération ne sera pas reconnue, survivra, et se nommera au compte rendu.
/// C'est le moindre mal — la spec préfère laisser survivre que d'effacer au jugé.
pub fn fichiers_du_livrable(cle: &str, images: &[String]) -> Vec<String> {
    let mut v: Vec<String> = [
        nom(cle, "interieur", "typ"),
        nom(cle, "interieur", "pdf"),
        nom(cle, "couverture", "typ"),
        nom(cle, "couverture", "pdf"),
        nom(cle, "couverture", "png"),
        "televersement.txt".to_string(),
    ]
    .into();
    v.extend(images.iter().cloned());
    v
}

/// Efface ce que l'application a écrit pour ce livrable, puis le répertoire s'il est vide.
///
/// **Sélectif et non récursif** (spec § 3) : l'effacement sans condition emporterait sans
/// recours ce qu'on aurait déposé là. Un fichier déjà parti n'est pas une erreur ; tout autre
/// échec refuse, comme `ebook::efface` le fait — un fichier qui résiste à la suppression est
/// exactement celui qu'une panne laisserait en place sous le nom du livre.
pub fn effacer_livrable(dossier: &Path, cle: &str, images: &[String]) -> Result<Nettoyage, String> {
    let mut n = Nettoyage::default();
    for f in fichiers_du_livrable(cle, images) {
        match std::fs::remove_file(dossier.join(&f)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => n.absents.push(f),
            Err(e) => return Err(format!("{f} ne s'efface pas : {e}")),
        }
    }
    // Ce qui reste porte le répertoire : on le lit avant de tenter de le retirer, pour pouvoir
    // le nommer. Un répertoire absent n'a rien laissé ; tout autre échec de lecture (permissions)
    // refuse plutôt que de se faire passer pour un répertoire vide.
    let reste = match std::fs::read_dir(dossier) {
        Ok(reste) => reste,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(n),
        Err(e) => return Err(format!("{} illisible : {e}", dossier.display())),
    };
    for e in reste.flatten() {
        n.etrangers
            .push(e.file_name().to_string_lossy().into_owned());
    }
    n.etrangers.sort();
    if n.etrangers.is_empty() {
        std::fs::remove_dir(dossier).map_err(|e| {
            format!(
                "le répertoire ne se retire pas ({}) : {e}",
                dossier.display()
            )
        })?;
        n.dossier_retire = true;
    }
    Ok(n)
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
            dos_texte_pages: None,
            pod_nom: "Essai".into(),
            format_nom: "Livre d'essai".into(),
            reliure_nom: "Broché d'essai".into(),
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

    /// Un PNG dont seul l'IHDR compte : `image::dimensions` ne décode rien d'autre.
    fn png(largeur: u32, hauteur: u32) -> Vec<u8> {
        let mut o = b"\x89PNG\r\n\x1a\n".to_vec();
        o.extend(13u32.to_be_bytes());
        o.extend(b"IHDR");
        o.extend(largeur.to_be_bytes());
        o.extend(hauteur.to_be_bytes());
        o.extend([8, 6, 0, 0, 0]);
        o
    }

    /// **La taille de l'objet entre dans la mesure, comme le recadrage pour la
    /// couverture.** La même main, posée sur le quart de la page, est quatre fois plus
    /// définie qu'étalée sur toute sa largeur — et c'est le second réglage qu'aucun
    /// aperçu ne trahit avant l'exemplaire reçu.
    #[test]
    fn la_taille_d_un_envoi_entre_dans_la_resolution_relevee() {
        let mut p = projet_en_images(Some("Léa.png"));
        p.images_envois.insert("Léa.png".into(), png(600, 400));
        let mesure = |taille: f64| {
            let mut e = p.meta.envois.liste[0].clone();
            e.place.taille = taille;
            image_d_envoi_pauvre(&p, &e, 135.0)
        };

        assert_eq!(mesure(0.25), None, "600 px sur 33,75 mm : 451 ppp");
        let a = mesure(1.0).expect("600 px sur 135 mm : 113 ppp, un avertissement");
        assert!(a.contains("Léa.png"), "{a}");
    }

    /// Un envoi manuscrit n'a pas d'image, et une image annoncée peut manquer de
    /// l'archive : ni l'un ni l'autre n'est un défaut de résolution. Le refus de
    /// composer, lui, est déjà porté par `trace`.
    #[test]
    fn un_envoi_sans_image_lisible_ne_se_mesure_pas() {
        let p = projet_en_images(Some("Léa.png"));
        // L'archive ne porte que quatre octets : pas de dimensions à lire.
        assert_eq!(
            image_d_envoi_pauvre(&p, &p.meta.envois.liste[0], 135.0),
            None
        );

        let manuscrit = crate::envoi::Envoi {
            main: crate::envoi::Main::Police {
                police: "Caveat".into(),
            },
            ..p.meta.envois.liste[0].clone()
        };
        assert_eq!(image_d_envoi_pauvre(&p, &manuscrit, 135.0), None);
    }

    /// Un package abouti, tel qu'`assembler` le rend : de quoi éprouver la fiche sans
    /// composer. Les chiffres sont ceux d'un vrai livrable BoD de 164 pages.
    fn package_d_essai() -> Package {
        Package {
            cle: "bod-135x215-broche-creme-90".into(),
            libelle: "BoD — 13,5 × 21,5 cm".into(),
            papier: "Crème 90 g".into(),
            pages: 164,
            gouttiere: 25.4,
            blanche: true,
            dos: 10.9125,
            dos_requis: None,
            fond_perdu: 3.175,
            planche: (238.8625, 181.35),
            chemins: vec![
                "/livres/LHC/bod/interieur-bod-135x215-broche-creme-90.pdf".into(),
                "/livres/LHC/bod/couverture-bod-135x215-broche-creme-90.pdf".into(),
            ],
            vignette: "/livres/LHC/bod/couverture-bod-135x215-broche-creme-90.png".into(),
            polices_introuvables: vec![],
            avertissements: vec![],
            interieur_partage: false,
        }
    }

    fn provider_bod() -> Provider {
        crate::catalogue::resout(&crate::catalogue::Fabrication {
            pod: "bod".into(),
            format: "135x215".into(),
            reliure: "broche".into(),
            papier: "creme-90".into(),
        })
        .unwrap()
        .provider()
    }

    /// **Ce que le dossier livré ne disait pas.** Les PDF sont muets : ils ne portent ni
    /// le format commandé, ni le papier, ni le dos qu'ils supposent. Tout cela vivait à
    /// l'écran, et le formulaire de l'imprimeur se remplissait de mémoire.
    ///
    /// Les noms sont ceux du catalogue, jamais ceux du COOKBOOK : celui-ci est de la
    /// prose relue par un humain, et en dupliquer les tableaux en données créerait deux
    /// vérités qui divergeraient au premier changement de guide.
    #[test]
    fn la_fiche_porte_ce_qu_il_faut_saisir_chez_l_imprimeur() {
        let f = fiche(
            &livre_au_dos_ecrit(),
            &provider_bod(),
            Some("Pelliculage mat"),
            &package_d_essai(),
        );

        assert!(f.starts_with("Les Heures creuses — Ivan Pjig\n"), "{f}");
        for attendu in [
            "Imprimeur        BoD",
            "Format           13,5 × 21,5 cm",
            "Reliure          Broché — dos carré collé",
            "Papier           Crème 90 g",
            "Finition         Pelliculage mat",
            "Pages            164 (dont une blanche de parité)",
            "Dos              10,91 mm",
            "Gouttière        25,4 mm",
            "Fond perdu       3,175 mm",
            "Planche          238,86 × 181,35 mm",
            "Fichiers         interieur-bod-135x215-broche-creme-90.pdf",
            "                 couverture-bod-135x215-broche-creme-90.pdf",
        ] {
            assert!(f.contains(attendu), "ligne « {attendu} » absente de :\n{f}");
        }
        assert!(
            f.contains("Le papier commandé doit être celui déclaré"),
            "{f}"
        );
    }

    /// Une ligne sans contenu ne paraît pas, comme la finition à l'écran : une entrée
    /// vide sur un document qu'on lit devant un formulaire se lirait comme un réglage
    /// manqué. La blanche de parité suit la même règle — elle ne se dit que là où il y
    /// en a une.
    #[test]
    fn une_ligne_sans_contenu_ne_parait_pas_sur_la_fiche() {
        let mut p = package_d_essai();
        p.blanche = false;
        let f = fiche(&livre_au_dos_ecrit(), &provider_bod(), None, &p);

        assert!(!f.contains("Finition"), "{f}");
        assert!(f.contains("Pages            164\n"), "{f}");
        assert!(!f.contains("blanche"), "{f}");
    }

    /// **La fiche dit ce que l'écran disait.** Un dossier relu trois mois plus tard n'a
    /// plus la fenêtre sous les yeux : les avertissements de la composition doivent
    /// voyager avec les PDF, et mot pour mot — c'est pour cela qu'ils sont écrits une
    /// fois, en Rust, et recopiés ici.
    #[test]
    fn la_fiche_recopie_les_avertissements_et_les_polices_de_repli() {
        let mut p = package_d_essai();
        p.avertissements = vec!["Image « couverture.png » posée à 91 ppp.".into()];
        p.polices_introuvables = vec!["plume ivan".into()];
        let f = fiche(&livre_au_dos_ecrit(), &provider_bod(), None, &p);

        assert!(
            f.contains("Image « couverture.png » posée à 91 ppp."),
            "{f}"
        );
        assert!(f.contains("plume ivan"), "{f}");
        assert!(f.contains("repli"), "{f}");
        // **En queue de fiche, après la consigne sur le papier.** Ce qui se lit avant
        // elle est ce qu'on saisit chez l'imprimeur ; ce qui la suit est ce qu'on décide
        // avant de téléverser. Intercalés, les avertissements couperaient en deux ce
        // qu'on recopie.
        let consigne = f.find("Le papier commandé").expect("la consigne du papier");
        assert!(consigne < f.find("plume ivan").unwrap(), "{f}");
        assert!(consigne < f.find("91 ppp").unwrap(), "{f}");
    }

    /// La cible d'un livrable d'essai : le premier papier du provider, aucun relevé,
    /// aucune finition. La clé de gabarit n'est plus à donner — c'est celle du provider.
    fn cible_d_essai(pr: &Provider, cle: &str) -> Cible {
        Cible {
            pr: pr.clone(),
            papier: pr.papiers[0].clone(),
            releve: Releve::default(),
            finition: None,
            cle: cle.into(),
        }
    }

    /// Le `Provider` d'essai, doté du seuil de texte au dos qu'un POD publierait.
    fn provider_au_seuil(seuil: Option<u32>) -> Provider {
        Provider {
            dos_texte_pages: seuil,
            ..provider_d_essai()
        }
    }

    /// Un livre dont le dos a de quoi composer : sans titre ni auteur, `composes` ne
    /// rend rien et le contrôle du dos n'aurait rien à juger.
    fn livre_au_dos_ecrit() -> crate::projet::Livre {
        crate::projet::Livre {
            titre: "Les Heures creuses".into(),
            auteur: "Ivan Pjig".into(),
            ..crate::projet::Livre::vide()
        }
    }

    fn gabarit_d_essai(pr: &Provider, pages: u32) -> Gabarit {
        Gabarit::pour(pr, &pr.papiers[0], pages, Releve::default()).unwrap()
    }

    fn photo(largeur: u32, hauteur: u32) -> Ressource {
        Ressource {
            fichier: "couverture.jpg".into(),
            largeur,
            hauteur,
        }
    }

    /// Sous le seuil publié, un dos qui compose quelque chose se signale : le texte y
    /// sera imprimé sur une tranche que l'imprimeur ne garantit pas droite, et cela ne
    /// se découvre aujourd'hui que sur l'exemplaire reçu.
    #[test]
    fn un_dos_qui_compose_sous_le_seuil_avertit() {
        let a = dos_sous_le_seuil(&provider_au_seuil(Some(81)), 64, true)
            .expect("64 pages sous un seuil de 81, dos composé : un avertissement");
        assert!(a.contains("64"), "{a}");
        assert!(a.contains("81"), "{a}");
        assert!(
            a.contains("Essai"),
            "l'avertissement nomme l'imprimeur : {a}"
        );
    }

    /// Un dos nu sous le seuil ne pose aucun problème : c'est le **texte** que
    /// l'imprimeur refuse sur une tranche mince, pas la tranche. Avertir ici enverrait
    /// chercher un réglage à éteindre qui l'est déjà.
    #[test]
    fn un_dos_nu_sous_le_seuil_ne_dit_rien() {
        assert_eq!(
            dos_sous_le_seuil(&provider_au_seuil(Some(81)), 64, false),
            None
        );
    }

    /// Le seuil est la pagination minimale **autorisée** : à 81 pages exactement, Lulu
    /// autorise. Avertir là serait refuser ce que le guide permet.
    #[test]
    fn au_seuil_exact_le_texte_au_dos_est_autorise() {
        assert_eq!(
            dos_sous_le_seuil(&provider_au_seuil(Some(81)), 81, true),
            None
        );
        assert!(dos_sous_le_seuil(&provider_au_seuil(Some(81)), 80, true).is_some());
    }

    /// L'absence de seuil vaut silence, et non zéro : quatre imprimeurs sur six n'en
    /// publient pas, et un contrôle inventé serait pire que pas de contrôle.
    #[test]
    fn un_pod_sans_seuil_ne_controle_rien() {
        assert_eq!(dos_sous_le_seuil(&provider_au_seuil(None), 24, true), None);
    }

    /// Une image dont on connaît les pixels et les millimètres : 1000 px sur 200 mm de
    /// large font 127 ppp, moins de la moitié de ce qu'une impression réclame. Le PDF se
    /// compose quand même — c'est un jugement d'auteur, pas une erreur —, mais rien
    /// aujourd'hui ne le dit avant l'exemplaire reçu.
    #[test]
    fn une_image_trop_pauvre_pour_l_impression_avertit() {
        let a = image_pauvre("quatrieme.jpg", 1000, 200.0)
            .expect("1000 px sur 200 mm : 127 ppp, un avertissement");
        assert!(a.contains("quatrieme.jpg"), "l'image est nommée : {a}");
        assert!(a.contains("127"), "la résolution mesurée est dite : {a}");
        assert!(a.contains("300"), "le seuil est dit : {a}");
    }

    /// Le seuil est atteint, pas dépassé : 300 px sur un pouce font exactement 300 ppp,
    /// et avertir là ferait douter d'une image qui convient.
    #[test]
    fn une_image_juste_au_seuil_ne_dit_rien() {
        assert_eq!(image_pauvre("une.png", 300, 25.4), None);
        assert!(image_pauvre("une.png", 299, 25.4).is_some());
    }

    /// **Ce que le contrôle de résolution a de particulier** : il mesure les pixels sur
    /// les millimètres où ils tombent, pas sur ceux de la zone. La même photo, cadrée
    /// deux fois plus grand, n'imprime que la moitié de sa définition — et c'est
    /// exactement le cas qu'on ne peut pas voir à l'écran avant l'exemplaire reçu.
    #[test]
    fn le_recadrage_entre_dans_la_resolution_relevee() {
        let pr = provider_au_seuil(None);
        let g = gabarit_d_essai(&pr, 244);
        let photo = photo(2000, 3000);
        let dit = |cv: &crate::couverture::Couverture| {
            avertissements(&livre_au_dos_ecrit(), cv, &pr, &g, 244, Some(&photo), None)
        };

        let cv = crate::maquettes::fournie("bandeau");
        assert!(
            dit(&cv).is_empty(),
            "2000 px sur cette zone dépassent 300 ppp : {:?}",
            dit(&cv)
        );

        let zoomee = crate::couverture::Couverture {
            cadrage: crate::image::Cadrage {
                zoom: 2.0,
                ..cv.cadrage
            },
            ..cv.clone()
        };
        let a = dit(&zoomee);
        assert_eq!(a.len(), 1, "{a:?}");
        assert!(a[0].contains("couverture.jpg"), "{}", a[0]);
    }

    /// Le prolongement panoramique compose la même photo sur la 4ème, le dos et la 1ère.
    /// Elle est pauvre une fois, pas trois : trois fois la même phrase ferait chercher
    /// trois images là où il n'y en a qu'une.
    #[test]
    fn une_image_posee_sur_deux_faces_n_avertit_qu_une_fois() {
        let pr = provider_au_seuil(None);
        let g = gabarit_d_essai(&pr, 244);
        let mut cv = crate::maquettes::fournie("bandeau");
        cv.quatrieme.fond = crate::couverture::FondQuatre::Panorama;

        let a = avertissements(
            &livre_au_dos_ecrit(),
            &cv,
            &pr,
            &g,
            244,
            Some(&photo(600, 900)),
            None,
        );
        assert_eq!(a.len(), 1, "{a:?}");
    }

    /// Le contrôle du dos est câblé sur ce que le dos **compose**, et non sur ce que la
    /// maquette déclare : une maquette dont tous les éléments de dos sont éteints ne
    /// réclame rien, et l'avertir enverrait éteindre ce qui l'est déjà.
    #[test]
    fn le_seuil_du_dos_se_juge_sur_ce_que_le_dos_compose() {
        let pr = provider_au_seuil(Some(81));
        let g = gabarit_d_essai(&pr, 64);
        let cv = crate::maquettes::fournie("bandeau");
        let a = avertissements(&livre_au_dos_ecrit(), &cv, &pr, &g, 64, None, None);
        assert_eq!(a.len(), 1, "{a:?}");
        assert!(a[0].contains("dos"), "{}", a[0]);

        // Les quatre textes que le dos peut porter, tous vides : `Livre::vide` en pose
        // trois par défaut, et un dos nu ne s'obtient qu'en les retirant.
        let muet = crate::projet::Livre {
            titre: String::new(),
            auteur: String::new(),
            editeur: String::new(),
            collection: String::new(),
            ..crate::projet::Livre::vide()
        };
        assert!(
            avertissements(&muet, &cv, &pr, &g, 64, None, None).is_empty(),
            "sans titre ni auteur, le dos ne compose rien"
        );
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

    /// Le catalogue ne promet jamais plus de pages que KDP n'en imprime.
    ///
    /// KDP publie son plafond par **couple** format × papier ; le catalogue, lui, croise
    /// trois axes qu'il tient pour indépendants — la reliure, le format, le papier. Les
    /// deux ne se recouvrent pas : le crème plafonne 52 pages sous le blanc sur la
    /// plupart des formats, mais 40 sous lui seulement en 8,5 × 8,5, et aucun jeu de
    /// bornes séparées ne reproduit la table publiée exactement. L'arbitrage est de ne
    /// jamais sur-promettre — le crème tombe juste sur les seize formats, et c'est le
    /// blanc qui se sous-borne, sur les cinq dont le plafond descend.
    ///
    /// Ce que ce contrôle protège : sans lui, l'application compose une couverture et son
    /// dos pour un livre que l'imprimeur refusera à la commande. L'erreur ne se voit sur
    /// aucun aperçu — le dos est juste, l'intérieur est juste, seul le bon de commande
    /// dira non.
    ///
    /// La table est indexée sur les formats **du pod** : un format ajouté sans son
    /// plafond publié fait échouer ce test au lieu de passer inaperçu.
    #[test]
    fn aucun_livrable_kdp_ne_promet_plus_de_pages_que_kdp_n_en_imprime() {
        // (format, plafond en blanc, plafond en crème) — page « Set Trim Size, Bleed, and
        // Margins », tableau des paginations du broché de kdp.amazon.com.
        const PUBLIE: [(&str, u32, u32); 16] = [
            ("5x8", 828, 776),
            ("506x781", 828, 776),
            ("525x8", 828, 776),
            ("55x85", 828, 776),
            ("6x9", 828, 776),
            ("614x921", 828, 776),
            ("669x961", 828, 776),
            ("7x10", 828, 776),
            ("744x969", 828, 776),
            ("75x925", 828, 776),
            ("8x10", 828, 776),
            ("825x6", 800, 750),
            ("825x825", 800, 750),
            ("85x85", 590, 550),
            ("85x11", 590, 550),
            ("827x1169", 780, 730),
        ];

        let kdp = crate::catalogue::pod("kdp").expect("le catalogue fournit KDP");
        for f in &kdp.formats {
            let (_, blanc, creme) = PUBLIE
                .iter()
                .find(|(cle, ..)| *cle == f.cle)
                .unwrap_or_else(|| {
                    panic!(
                        "kdp / {} : format absent de la table publiée de ce test — \
                         y porter son plafond relevé chez KDP avant de l'ajouter au pod.",
                        f.cle
                    )
                });

            for (papier, publie) in [("blanc", blanc), ("creme", creme)] {
                let fab = crate::catalogue::Fabrication {
                    pod: "kdp".into(),
                    format: f.cle.clone(),
                    reliure: "broche".into(),
                    papier: papier.into(),
                };
                let r = crate::catalogue::resout(&fab).expect("livrable KDP résolu");
                let pr = r.provider();

                assert!(
                    verifie_pagination(&fab.cle(), publie + 2, &pr, r.papier).is_err(),
                    "{} / {papier} : {} pages passent, alors que KDP n'en imprime que {publie}",
                    f.cle,
                    publie + 2
                );
            }

            // Sans quoi un catalogue qui refuserait tout passerait ce test au vert. C'est
            // le crème que la politique promet exact : lui seul est ancré par le bas.
            let fab = crate::catalogue::Fabrication {
                pod: "kdp".into(),
                format: f.cle.clone(),
                reliure: "broche".into(),
                papier: "creme".into(),
            };
            let r = crate::catalogue::resout(&fab).expect("livrable KDP résolu");
            let pr = r.provider();
            assert!(
                verifie_pagination(&fab.cle(), *creme, &pr, r.papier).is_ok(),
                "{} / crème : {creme} pages refusées, alors que KDP les imprime",
                f.cle
            );
        }
    }

    /// Le `Livre` du témoin, réduit à ce qu'`assembler` et `composer_interieur` lisent.
    fn livre_d_essai() -> crate::projet::Livre {
        crate::projet::Livre {
            isbn: String::new(),
            depot_legal: String::new(),
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
            &cible_d_essai(&pr, "essai"),
            &pret,
            dir.path(),
            &Typst::new("typst-qui-n-existe-pas"),
        )
        .unwrap_err();
        assert!(e.contains("maquette"), "l'intérieur a été recomposé : {e}");
    }

    /// **Un dossier livré dit ce qu'il contient, lui et pas un autre.** L'exemplaire
    /// d'un dédicataire porte son propre intérieur et sa propre fiche : recopier celle
    /// de la référence y mettrait les avertissements d'un autre exemplaire — ou pas les
    /// siens. Composition réelle, parce que c'est le seul chemin qui passe par
    /// `assembler_envois`.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn chaque_exemplaire_d_envoi_porte_sa_propre_fiche() {
        let mut projet = Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        projet.meta.couverture.maquette = Some(
            crate::maquettes::par_cle(None, "filets")
                .expect("maquette fournie « filets »")
                .couverture,
        );
        projet.meta.envois.liste = ["Léa", "Rex"]
            .into_iter()
            .map(|qui| crate::envoi::Envoi {
                dedicataire: qui.into(),
                contenu: format!("À {qui}."),
                main: crate::envoi::Main::Police {
                    police: "Caveat".into(),
                },
                ..Default::default()
            })
            .collect();

        let racine = tempfile::tempdir().unwrap();
        let typst = Typst::new("typst")
            .avec_polices(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"));
        let pr = provider_d_essai();
        let sorties = assembler_envois(
            &projet,
            &Cible {
                finition: Some("Pelliculage d'essai".into()),
                ..cible_d_essai(&pr, "essai-livre-broche-creme")
            },
            racine.path(),
            &typst,
        )
        .expect("les deux exemplaires");

        assert_eq!(sorties.len(), 2);
        for (dossier, p) in &sorties {
            let fiche = racine.path().join(dossier).join("televersement.txt");
            let lu = std::fs::read_to_string(&fiche)
                .unwrap_or_else(|e| panic!("{} : {e}", fiche.display()));
            assert!(lu.contains("Pelliculage d'essai"), "{lu}");
            assert!(
                p.chemins.iter().any(|c| c.ends_with("televersement.txt")),
                "la fiche doit paraître au compte rendu : {:?}",
                p.chemins
            );
        }
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
        // Chaque cible porte **son** `Provider`, comme `commands::packager` les fabrique
        // par `Resolu::provider` : deux vues plates d'un même gabarit qui ne diffèrent
        // que par le papier de leur fabrication. Le même `pr` des deux côtés cachait le
        // seul cas où la mémoïsation avait à distinguer clé de gabarit et clé de
        // livrable — et laissait passer une génération qui plantait sur les vrais.
        let vue_plate = |papier: &str| Provider {
            fabrication: crate::catalogue::Fabrication {
                papier: papier.into(),
                ..pr.fabrication.clone()
            },
            ..pr.clone()
        };
        let cibles = [
            Cible {
                papier: creme,
                ..cible_d_essai(&vue_plate("creme"), "essai-livre-broche-creme")
            },
            Cible {
                papier: blanc,
                ..cible_d_essai(&vue_plate("blanc-essai"), "essai-livre-broche-blanc-essai")
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
        let c = |cle: &str| cible_d_essai(&pr, cle);
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

    /// Le cas qui faisait planter la génération : deux livrables du même gabarit sur
    /// deux papiers différents. `Resolu::provider` pose la clé sur le triplet mais garde
    /// la fabrication du livrable, papier compris — les deux `Provider` ne diffèrent donc
    /// que par ce champ, et c'est exactement le lot que `lot` mémoïse : un intérieur, N
    /// papiers. Comparer les `Provider` entiers refusait ce cas nominal.
    #[test]
    fn deux_papiers_du_meme_gabarit_font_le_meme_gabarit() {
        let creme = provider_d_essai();
        let blanc = Provider {
            fabrication: crate::catalogue::Fabrication {
                papier: "blanc".into(),
                ..creme.fabrication.clone()
            },
            ..creme.clone()
        };
        assert!(meme_gabarit(&creme, &blanc));
    }

    /// L'autre bord, sans quoi rendre `true` toujours suffirait : une même clé posée sur
    /// deux géométries n'est pas le même gabarit, et l'intérieur du premier ne peut pas
    /// être resservi au second.
    #[test]
    fn une_geometrie_differente_n_est_pas_le_meme_gabarit() {
        let a = provider_d_essai();
        let b = Provider {
            format: (148.0, 210.0),
            ..a.clone()
        };
        assert!(!meme_gabarit(&a, &b));
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

    /// Un projet dont l'unique livrable est généré, à jour, et dont les deux fichiers d'intérieur
    /// sont sur le disque : le cas nominal de la réutilisation.
    ///
    /// La fabrication est celle du catalogue réel (celle de `Projet::nouveau`) : `empreinte`
    /// résout son gabarit, et le test vaut donc sur le même chemin que l'application.
    fn projet_genere(racine: &std::path::Path, pages: u32) -> (crate::projet::Projet, String) {
        let mut projet =
            crate::projet::Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        let l = projet.meta.livraison.livrables[0].clone();
        let cle = l.cle();
        projet.meta.livraison.retenir_mesure(
            &l.fabrication.cle_gabarit(),
            crate::projet::Mesure {
                pages,
                gouttiere: 14.0,
                blanche: false,
                empreinte: None,
                polices_introuvables: vec![],
            },
        );
        let dossier = racine.join(&cle);
        std::fs::create_dir_all(&dossier).unwrap();
        std::fs::write(dossier.join(nom(&cle, "interieur", "typ")), b"source").unwrap();
        std::fs::write(dossier.join(nom(&cle, "interieur", "pdf")), b"%PDF-faux").unwrap();
        let empreinte = crate::empreinte::interieur(&projet, &l);
        projet.meta.livraison.livrables[0].generation = crate::projet::Generation::Fait {
            interieur: empreinte,
            couverture: "peu importe".into(),
        };
        (projet, cle)
    }

    /// Le cas nominal : un livrable généré et à jour prête son intérieur, avec la pagination que
    /// le projet a retenue — jamais relue dans le PDF, qui ne la porte pas.
    #[test]
    fn un_interieur_a_jour_sur_le_disque_se_prete() {
        let racine = tempfile::tempdir().unwrap();
        let (projet, cle) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();

        let i = interieur_du_disque(&projet, racine.path(), &fab, &[])
            .expect("l'intérieur du livrable généré");
        assert_eq!(i.pages, 266);
        assert_eq!(
            i.pdf,
            racine.path().join(&cle).join(nom(&cle, "interieur", "pdf"))
        );
    }

    /// **Le verdict qui fait que Régénérer régénère.** Une cible de la passe ne se prête pas son
    /// propre intérieur : elle copierait son PDF sur lui-même, s'annoncerait `interieur_partage`,
    /// et rien n'aurait été recomposé. C'est aussi ce qui laisse « Tout regénérer » composer comme
    /// avant, toutes ses cibles étant exclues.
    #[test]
    fn une_cible_de_la_passe_ne_se_prete_pas_son_propre_interieur() {
        let racine = tempfile::tempdir().unwrap();
        let (projet, cle) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();

        assert!(
            interieur_du_disque(&projet, racine.path(), &fab, &[&cle]).is_none(),
            "régénérer se serait prêté son propre intérieur : il n'aurait rien recomposé"
        );
    }

    /// **La clé de partage d'un lot sépare deux papiers.** C'est elle qui décide qui
    /// compose et qui copie dans `lot`, et elle sort de `Cible::fabrication` — le seul
    /// endroit du fichier qui sache que le papier d'une cible n'est pas celui que son
    /// `Provider` porte par défaut. Prendre `pr.fabrication` telle quelle rangerait les
    /// deux papiers sous une même clé, et le second copierait le PDF du premier.
    ///
    /// Éprouvé ici et non dans `lot`, pour la même raison que `meme_gabarit` l'est :
    /// `lot` compose, donc réclame Typst.
    #[test]
    fn la_cle_de_partage_separe_deux_papiers() {
        let pr = provider_d_essai();
        // Le second papier est bâti ici plutôt qu'ajouté au provider d'essai : ce dernier
        // sert à une trentaine de tests que ce bord ne regarde pas.
        let second = Papier {
            cle: "photo".into(),
            nom: "Photo d'essai".into(),
            ..pr.papiers[0].clone()
        };
        let une = Cible {
            papier: pr.papiers[0].clone(),
            ..cible_d_essai(&pr, "essai")
        };
        let autre = Cible {
            papier: second,
            ..cible_d_essai(&pr, "essai")
        };

        assert_eq!(
            une.fabrication().cle_gabarit(),
            autre.fabrication().cle_gabarit(),
            "les deux papiers doivent bien partager un gabarit, sinon le test ne dit rien"
        );
        assert_ne!(
            une.fabrication().cle(),
            autre.fabrication().cle(),
            "deux papiers partagent une clé de partage : le second copierait le PDF du premier"
        );
    }

    /// **Un papier ne prête pas son intérieur à un autre papier.** Le partage se faisait par
    /// gabarit, et il était juste tant que rien du papier n'entrait dans l'intérieur. Depuis
    /// `%PAPIER%`, il ne l'est plus : le second livrable recevrait le PDF du premier, qui
    /// nomme l'autre papier. C'est le pendant, dans le chemin de génération, de ce que
    /// `empreinte::deux_papiers_du_meme_gabarit_ne_partagent_pas_une_empreinte` protège dans
    /// celui de la péremption — et l'empreinte seule ne l'attrape pas, ce prêt-ci ne
    /// comparant que la fraîcheur du prêteur.
    #[test]
    fn un_papier_ne_prete_pas_son_interieur_a_un_autre_papier() {
        let racine = tempfile::tempdir().unwrap();
        let (projet, _) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();

        let mut autre = fab.clone();
        let pod = crate::catalogue::resout(&fab).expect("livrable d'essai résolu");
        let papiers = &pod.provider().papiers;
        assert!(
            papiers.len() > 1,
            "le livrable d'essai n'offre qu'un papier : le bord ne peut pas se tester"
        );
        autre.papier = papiers[1].cle.clone();
        assert_eq!(
            fab.cle_gabarit(),
            autre.cle_gabarit(),
            "les deux papiers doivent bien partager un gabarit, sinon le test ne dit rien"
        );

        assert!(
            interieur_du_disque(&projet, racine.path(), &autre, &[]).is_none(),
            "l'intérieur d'un papier s'est prêté à un autre : le second porterait le nom du premier"
        );
        assert!(
            interieur_du_disque(&projet, racine.path(), &fab, &[]).is_some(),
            "le même papier doit toujours se prêter, sinon ce n'est plus un prêt"
        );
    }

    /// Une empreinte qui a bougé ne prête rien : c'est tout le mécanisme du lot 1. Le PDF est là,
    /// il est lisible, et il ne compose plus ce livre-là.
    #[test]
    fn un_interieur_perime_ne_se_prete_pas() {
        let racine = tempfile::tempdir().unwrap();
        let (mut projet, _) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();
        projet.remplacer_texte("## 01 - Un\n\nUn autre paragraphe.".into());
        // `remplacer_texte` oublie les mesures : on remet celle du gabarit pour prouver que c'est
        // bien l'empreinte, et non l'absence de mesure, qui refuse le prêt.
        projet.meta.livraison.retenir_mesure(
            &fab.cle_gabarit(),
            crate::projet::Mesure {
                pages: 266,
                gouttiere: 14.0,
                blanche: false,
                empreinte: None,
                polices_introuvables: vec![],
            },
        );

        assert!(interieur_du_disque(&projet, racine.path(), &fab, &[]).is_none());
    }

    /// Un PDF effacé à la main ne se copie pas : la source seule laisserait dans le répertoire
    /// livré un `.typ` qui ne correspond à rien.
    #[test]
    fn un_fichier_manquant_ne_se_prete_pas() {
        let racine = tempfile::tempdir().unwrap();
        let (projet, cle) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();
        std::fs::remove_file(racine.path().join(&cle).join(nom(&cle, "interieur", "pdf"))).unwrap();

        assert!(interieur_du_disque(&projet, racine.path(), &fab, &[]).is_none());
    }

    /// Sans mesure, pas de pagination à prêter — et rien à inventer : `InterieurCompose` la porte,
    /// et le PDF ne la dit pas.
    #[test]
    fn sans_mesure_rien_ne_se_prete() {
        let racine = tempfile::tempdir().unwrap();
        let (mut projet, _) = projet_genere(racine.path(), 266);
        let fab = projet.meta.livraison.livrables[0].fabrication.clone();
        projet.meta.livraison.oublier_mesures();

        assert!(interieur_du_disque(&projet, racine.path(), &fab, &[]).is_none());
    }

    /// **Spec § 8 : la réutilisation vaut d'un appel à l'autre, pas seulement dans une passe.**
    /// C'est le risque nommé du chantier : générer à l'ajout déplace l'attente sur chaque ajout,
    /// et si l'amorce était mal câblée le symptôme serait un ajout lent que rien ne dit.
    ///
    /// La preuve ne peut pas être « les octets sont identiques » : une recomposition les rendrait
    /// identiques aussi. Le PDF du premier livrable est donc **marqué** entre les deux appels ; si
    /// le second porte la marque, il l'a copié, et rien n'a été recomposé. Marquer ne périme rien :
    /// les empreintes portent sur les données du projet, jamais sur le PDF.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn un_interieur_se_reutilise_d_un_appel_a_l_autre() {
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
            dos: crate::catalogue::Dos::Multiplie {
                par: 0.08,
                plus: 2.0,
            },
            pages: None,
            source: None,
        };
        let vue_plate = |papier: &str| Provider {
            fabrication: crate::catalogue::Fabrication {
                papier: papier.into(),
                ..pr.fabrication.clone()
            },
            ..pr.clone()
        };
        let cle_a = "essai-livre-broche-creme";
        let cle_b = "essai-livre-broche-blanc-essai";

        // Les deux livrables sont dans le projet dès le départ : c'est là que l'amorce va les
        // chercher, et non dans les cibles.
        projet.meta.livraison.livrables = ["creme", "blanc-essai"]
            .into_iter()
            .map(|papier| {
                crate::projet::Livrable::pour(crate::catalogue::Fabrication {
                    papier: papier.into(),
                    ..pr.fabrication.clone()
                })
            })
            .collect();
        projet.meta.livraison.courant = cle_a.into();

        // Premier appel : une seule cible, le crème. Il compose.
        let a = lot(
            &projet,
            &[Cible {
                papier: creme,
                ..cible_d_essai(&vue_plate("creme"), cle_a)
            }],
            racine.path(),
            &typst,
        )
        .remove(0)
        .expect("le premier package");
        assert!(!a.interieur_partage, "le premier compose");

        // Ce que `commands::retenir` fait dans l'application : la mesure, puis les empreintes.
        let l = projet.meta.livraison.livrables[0].clone();
        projet.meta.livraison.retenir_mesure(
            &l.fabrication.cle_gabarit(),
            crate::projet::Mesure {
                pages: a.pages,
                gouttiere: a.gouttiere,
                blanche: a.blanche,
                empreinte: None,
                polices_introuvables: a.polices_introuvables.clone(),
            },
        );
        projet.meta.livraison.livrables[0].generation = crate::projet::Generation::Fait {
            interieur: crate::empreinte::interieur(&projet, &l),
            couverture: crate::empreinte::couverture(&projet, &l),
        };

        // La marque : si le second package la porte, il a copié.
        let pdf_a = racine
            .path()
            .join(cle_a)
            .join(nom(cle_a, "interieur", "pdf"));
        std::fs::write(&pdf_a, b"MARQUE-DU-PREMIER").unwrap();

        // Second appel, séparé : le blanc. Il doit copier.
        let b = lot(
            &projet,
            &[Cible {
                papier: blanc,
                ..cible_d_essai(&vue_plate("blanc-essai"), cle_b)
            }],
            racine.path(),
            &typst,
        )
        .remove(0)
        .expect("le second package");

        assert!(
            b.interieur_partage,
            "le second devait copier, pas recomposer"
        );
        assert_eq!(b.pages, a.pages, "la pagination vient de la mesure retenue");
        assert_eq!(
            std::fs::read(
                racine
                    .path()
                    .join(cle_b)
                    .join(nom(cle_b, "interieur", "pdf"))
            )
            .unwrap(),
            b"MARQUE-DU-PREMIER",
            "l'intérieur a été recomposé au lieu d'être copié"
        );
    }

    /// La liste de ce que l'application a écrit dans le répertoire d'un livrable : cinq noms
    /// tirés de la clé, la fiche qui n'en porte pas, et les images de couverture sous leur nom
    /// d'origine. C'est cette liste, et elle seule, que la suppression efface.
    #[test]
    fn les_fichiers_d_un_livrable_sont_ceux_que_l_application_a_ecrits() {
        let f = fichiers_du_livrable("lulu-108x175-broche-creme", &["une.jpg".to_string()]);
        for attendu in [
            "interieur-lulu-108x175-broche-creme.typ",
            "interieur-lulu-108x175-broche-creme.pdf",
            "couverture-lulu-108x175-broche-creme.typ",
            "couverture-lulu-108x175-broche-creme.pdf",
            "couverture-lulu-108x175-broche-creme.png",
            "televersement.txt",
            "une.jpg",
        ] {
            assert!(f.iter().any(|x| x == attendu), "{attendu} manque : {f:?}");
        }
        assert_eq!(f.len(), 7, "rien d'autre ne doit y être : {f:?}");
    }

    /// **Ce qu'on a déposé là survit.** L'effacement récursif sans condition emporterait sans
    /// recours un fichier qu'on aurait rangé dans ce répertoire — un bon de commande, une épreuve
    /// annotée. Il reste, le répertoire avec lui, et le compte rendu le nomme.
    #[test]
    fn un_fichier_etranger_survit_et_se_nomme() {
        let dir = tempfile::tempdir().unwrap();
        let cle = "essai-livre-broche-creme";
        for f in fichiers_du_livrable(cle, &["une.jpg".to_string()]) {
            std::fs::write(dir.path().join(f), b"x").unwrap();
        }
        std::fs::write(dir.path().join("bon-de-commande.pdf"), b"x").unwrap();

        let n = effacer_livrable(dir.path(), cle, &["une.jpg".to_string()]).unwrap();

        assert_eq!(n.etrangers, vec!["bon-de-commande.pdf".to_string()]);
        assert!(
            !n.dossier_retire,
            "le répertoire porte encore quelque chose"
        );
        assert!(dir.path().join("bon-de-commande.pdf").is_file());
        assert!(!dir.path().join(nom(cle, "interieur", "pdf")).exists());
        assert!(n.absents.is_empty(), "tous les fichiers connus étaient là");
    }

    /// Rien d'étranger : le répertoire s'en va avec ce qu'il portait. C'est le cas ordinaire, et
    /// laisser un répertoire vide sous le nom d'un livrable qui n'existe plus ferait douter de ce
    /// qui a été supprimé.
    #[test]
    fn un_repertoire_vide_apres_effacement_s_en_va() {
        let dir = tempfile::tempdir().unwrap();
        let livrable = dir.path().join("essai-livre-broche-creme");
        std::fs::create_dir_all(&livrable).unwrap();
        for f in fichiers_du_livrable("essai-livre-broche-creme", &[]) {
            std::fs::write(livrable.join(f), b"x").unwrap();
        }

        let n = effacer_livrable(&livrable, "essai-livre-broche-creme", &[]).unwrap();

        assert!(n.dossier_retire);
        assert!(!livrable.exists());
    }

    /// Un répertoire déjà parti — effacé à la main, ou jamais écrit parce que la génération avait
    /// échoué — n'est pas une erreur : le livrable s'en va, et le compte rendu dit ce qui n'était
    /// plus là. C'est l'arbitrage d'`ebook::efface`.
    #[test]
    fn un_repertoire_deja_parti_ne_fait_pas_echouer() {
        let dir = tempfile::tempdir().unwrap();
        let absent = dir.path().join("jamais-ecrit");

        let n = effacer_livrable(&absent, "essai-livre-broche-creme", &[]).unwrap();

        assert_eq!(
            n.absents.len(),
            6,
            "les six fichiers connus manquaient : {n:?}"
        );
        assert!(n.etrangers.is_empty());
        assert!(
            !n.dossier_retire,
            "il n'y avait pas de répertoire à retirer"
        );
    }

    /// Un répertoire présent mais illisible n'est pas « rien n'y est resté » : la confondre avec
    /// un répertoire absent ferait rendre `Ok` alors que le contenu n'a pas pu être vérifié — et
    /// la tâche 6 retire le livrable du projet dès que cette fonction rend `Ok`. Un fichier qui
    /// résiste doit refuser, comme `remove_file` le fait déjà pour les siens.
    #[test]
    #[cfg(unix)]
    fn un_repertoire_illisible_fait_echouer() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let livrable = dir.path().join("essai-livre-broche-creme");
        std::fs::create_dir_all(&livrable).unwrap();
        // Écriture et exécution sans lecture : les `remove_file` trouvent un répertoire vide
        // (NotFound), mais `read_dir` ne peut plus lister ce qu'il contient.
        std::fs::set_permissions(&livrable, std::fs::Permissions::from_mode(0o300)).unwrap();

        let resultat = effacer_livrable(&livrable, "essai-livre-broche-creme", &[]);

        // Remis lisible avant toute assertion : sinon un échec laisserait le `tempdir` sur des
        // permissions qui empêchent aussi sa propre suppression en fin de test.
        std::fs::set_permissions(&livrable, std::fs::Permissions::from_mode(0o700)).unwrap();

        assert!(
            resultat.is_err(),
            "un répertoire illisible doit refuser, pas se faire passer pour vide : {resultat:?}"
        );
    }
}
