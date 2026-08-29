//! Composition de l'intérieur : source Typst, et convergence gouttière/parité.
//!
//! Deux conditions doivent être satisfaites **ensemble** : la gouttière doit
//! correspondre à la tranche de pagination effective, et le compte de pages doit être
//! pair — une feuille porte deux pages, les imprimeurs refusent l'impair. Chacune
//! peut déplacer la pagination, d'où la reprise.
//!
//! Le compte de pages produit ici est celui que consomme la couverture pour calculer
//! le dos. Il ne transite par aucune saisie humaine : c'est la raison d'être de l'app.

use serde::{Deserialize, Serialize};

use crate::catalogue::Provider;
use crate::gabarit::Contexte;
use crate::manuscrit::{echappe, echappe_chaine, inline, Bloc, Piece, Sorte, SCENE};
use crate::projet::Livre;
use crate::typst::MARQUEUR;

/// Corps du texte, en points.
///
/// Il vivait dans la table des gabarits, **identique dans ses quatorze entrées** :
/// ce n'est pas un fait d'imprimeur mais un choix typographique. La pagination en dépend,
/// donc le dos : le déplacer est un acte délibéré, à revalider sur un livre réel.
pub const CORPS_PT: f64 = 9.5;

/// Interligne, en multiple du corps. Rapporté à `leading` Typst par `- 1.0`.
pub const INTERLIGNE: f64 = 1.42;

/// Corps du folio, en points.
pub const FOLIO_PT: f64 = 8.0;

/// Les polices que l'intérieur admet.
///
/// Volontairement plus courte que `couverture::POLICES` : ce sont les seules qui
/// tiennent trois cents pages de corps de texte, chacune avec un vrai italique. Un
/// titrage comme Oswald ferait un roman illisible, et l'erreur ne se découvrirait
/// qu'après tirage.
pub const POLICES_TEXTE: &[&str] = &[
    "EB Garamond",
    "Crimson Pro",
    "Alegreya",
    "Cardo",
    "Vollkorn",
    "Spectral",
    "Libre Baskerville",
];

fn police_defaut() -> String {
    "EB Garamond".into()
}

/// Les bornes qu'une taille d'intérieur ne franchit pas, en points.
///
/// Elles ne cherchent pas le bon goût — 4 pt est illisible et 48 pt grotesque, mais ni
/// l'un ni l'autre ne casse la composition. Elles gardent de ce qui la casse : un 0 ou
/// un négatif, que Typst compose sans lever d'erreur, en rendant un PDF blanc dont la
/// pagination donne un dos faux.
pub const MIN_PT: f64 = 4.0;
pub const MAX_PT: f64 = 48.0;

/// Où la table des matières se compose — ou pas du tout.
///
/// **Absente par défaut**, et pour la raison qui a éteint la collection sur le dos :
/// allumée d'office, elle ajouterait des pages à tous les livres déjà composés, donc
/// changerait leur dos sans que personne l'ait demandé.
///
/// Le réglage vit dans `Interieur` et non dans `Livre` : c'est un choix de composition,
/// qui déplace la pagination comme la police le fait, pas un trait de l'identité du
/// livre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Table {
    #[default]
    Absente,
    EnTete,
    EnFin,
}

/// Réglages d'intérieur du projet.
///
/// L'imprimeur impose le format, les marges et la gouttière ; le livre choisit son
/// caractère et ses tailles. C'est la raison pour laquelle rien de ceci n'est un champ
/// de `Provider` — le corps, en particulier, était identique dans les quatorze entrées
/// de la table.
///
/// **Chacune de ces tailles déplace la pagination, donc le dos** : `modifier_interieur`
/// oublie les mesures pour cette raison, comme il le fait pour la police.
///
/// `#[serde(default)]` porte sur la structure entière : un `.ozalid` écrit avant ces
/// champs les reçoit de `Default`, et lit donc le livre qu'il composait. `VERSION` n'a
/// pas à bouger — même dispositif que `titre_page`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Interieur {
    pub police: String,
    /// Où la table des matières se compose. L'allumer ajoute des pages, donc change le
    /// dos : `modifier_interieur` oublie les mesures pour cette raison, comme il le fait
    /// pour la police.
    pub table: Table,
    /// Le texte courant.
    pub corps: f64,
    /// Le faux-titre, seul sur la page 1.
    pub faux_titre: f64,
    /// L'auteur, en tête de la page de titre.
    pub page_titre_auteur: f64,
    /// Le titre, au milieu de la page de titre.
    pub page_titre_titre: f64,
    /// La mention de genre, sous le titre.
    pub page_titre_genre: f64,
    /// Le pavé de copyright, au bas de son verso.
    pub copyright: f64,
    /// La dédicace, quand le livre en porte une.
    pub dedicace: f64,
    /// Le numéro d'une page de partie **et** celui d'un chapitre : deux niveaux du même
    /// gabarit, un seul réglage.
    pub numero: f64,
    /// Le titre qui suit ce numéro, pour la partie comme pour le chapitre.
    pub titre_section: f64,
    /// Le titre d'une pièce à texte — préface, postface : le mot occupe la ligne du
    /// numéro, mais composé comme un titre, d'où sa taille à lui.
    pub ouverture_piece: f64,
    /// Une ligne de la table des matières — celle du titre de la table, lui, est
    /// `ouverture_piece` : la table s'ouvre comme une préface, c'est une pièce du livre.
    pub entree_table: f64,
    /// Le folio.
    pub folio: f64,
}

impl Default for Interieur {
    fn default() -> Self {
        Self {
            police: police_defaut(),
            table: Table::Absente,
            corps: CORPS_PT,
            faux_titre: 11.0,
            page_titre_auteur: 10.5,
            page_titre_titre: 15.0,
            page_titre_genre: 10.0,
            copyright: 8.0,
            dedicace: 9.5,
            numero: 13.0,
            titre_section: 10.0,
            ouverture_piece: 10.0,
            entree_table: 9.0,
            folio: FOLIO_PT,
        }
    }
}

impl Interieur {
    /// Les douze tailles, chacune sous le nom que l'interface lui donne.
    ///
    /// Une liste plutôt que onze conditions recopiées : le message d'erreur nomme le
    /// champ fautif, et un douzième réglage s'ajoute ici en une ligne.
    fn tailles(&self) -> [(&'static str, f64); 12] {
        [
            ("corps du texte", self.corps),
            ("faux-titre", self.faux_titre),
            ("auteur en page de titre", self.page_titre_auteur),
            ("titre en page de titre", self.page_titre_titre),
            ("genre en page de titre", self.page_titre_genre),
            ("copyright", self.copyright),
            ("dédicace", self.dedicace),
            ("numéro de partie ou de chapitre", self.numero),
            ("titre de partie ou de chapitre", self.titre_section),
            ("titre de préface ou de postface", self.ouverture_piece),
            ("entrée de table des matières", self.entree_table),
            ("folio", self.folio),
        ]
    }

    /// Refuse une police absente de la liste, et une taille hors bornes.
    ///
    /// Sans le premier contrôle, Typst composerait dans sa police par défaut **sans
    /// lever d'erreur** : `--ignore-system-fonts` empêche une substitution par le
    /// système, pas une substitution par le défaut du binaire. Sans le second, un 0
    /// tapé dans un champ passerait de même — le formulaire pose les mêmes bornes,
    /// mais un `.ozalid` retouché à la main ne passe pas par le formulaire.
    pub fn verifie(&self) -> Result<(), String> {
        if !POLICES_TEXTE.contains(&self.police.as_str()) {
            return Err(format!(
                "police d'intérieur inconnue : « {} ». Attendu : {}.",
                self.police,
                POLICES_TEXTE.join(", ")
            ));
        }
        for (quoi, pt) in self.tailles() {
            if !(MIN_PT..=MAX_PT).contains(&pt) {
                return Err(format!(
                    "taille du {quoi} hors bornes : {pt} pt. Attendu entre {MIN_PT} et {MAX_PT} pt."
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reglage {
    pub gouttiere: f64,
    /// Page blanche de fin, sans folio, pour ramener le compte à un nombre pair.
    pub blanche: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resultat {
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
}

/// Nombre de reprises avant d'admettre que la composition n'a pas de point fixe.
/// La bascule de parité converge en un tour puisqu'elle change le compte de 1
/// exactement ; il ne reste à absorber que les changements de tranche.
const REPRISES: usize = 4;

/// Cherche le réglage stable, en ne mesurant que le compte de pages.
///
/// `mesure` compose et rend le compte, sans produire de PDF : la convergence ne coûte
/// donc aucun fichier jeté. Elle est injectée pour que la boucle soit testable sans
/// binaire Typst — c'est de la logique métier, pas de l'orchestration de processus.
pub fn converge(
    pr: &Provider,
    mut mesure: impl FnMut(&Reglage) -> Result<u32, String>,
) -> Result<Resultat, String> {
    let mut r = Reglage {
        // Hypothèse de départ : la première tranche du gabarit.
        gouttiere: pr.gouttieres[0].2,
        blanche: false,
    };
    for _ in 0..REPRISES {
        let pages = mesure(&r)?;
        // Sort proprement si la tranche est inconnue, plutôt que d'inventer.
        let g = pr.gouttiere(pages)?;
        if (g - r.gouttiere).abs() > f64::EPSILON {
            r.gouttiere = g;
            continue;
        }
        if pages % 2 == 1 {
            r.blanche = !r.blanche;
            continue;
        }
        return Ok(Resultat {
            pages,
            gouttiere: r.gouttiere,
            blanche: r.blanche,
        });
    }
    Err("la composition ne converge pas (gouttière ou parité oscillantes).".into())
}

/// Ce qu'un envoi dépose sur sa page.
///
/// `interieur` ne connaît ni la main du livre, ni d'où l'image vient : il reçoit ce
/// que l'envoi a décidé. Une image écrite à la main et une image produite par un
/// modèle de diffusion arrivent ici de la même façon — ce module n'a pas à savoir
/// laquelle, seulement qu'elle est posée à côté de la source.
#[derive(Debug, Clone)]
pub enum Quoi<'a> {
    /// Un texte, composé dans la main de cet envoi.
    Texte { police: &'a str, texte: &'a str },
    /// Une image, déjà écrite à côté de la source, désignée par son seul nom.
    ///
    /// `Cow` parce que le nom écrit n'est pas toujours celui de l'archive : une photo
    /// détourée sort en PNG et change d'extension. L'emprunt subsiste quand rien n'est
    /// détouré, et c'est le cas des projets d'avant ce chantier.
    Image { fichier: std::borrow::Cow<'a, str> },
}

/// Un envoi et sa place sur la page.
#[derive(Debug, Clone)]
pub struct Trace<'a> {
    pub quoi: Quoi<'a>,
    pub place: &'a crate::envoi::Place,
}

/// Le rapport entre la largeur de l'objet et le corps de son écriture.
///
/// L'objet est self-similaire : l'agrandir agrandit les lettres, parce que tirer un
/// coin à la souris agrandit une signature — il n'élargit pas une colonne de texte
/// pour la laisser se recomposer. Le corps suit donc la taille.
///
/// La valeur cale le nouveau réglage sur l'ancien : jusqu'à la v4, l'envoi se composait
/// en 14 pt dans un bloc de 70 % de la justification. Sur une page de 127 mm, une
/// taille de 0,60 donne 76,2 mm de large, et 14 pt valent 4,94 mm — d'où 4,94 / 76,2.
const CORPS_SUR_LARGEUR: f64 = 0.0648;

/// Source Typst complète de l'intérieur.
fn assemble(
    ctx: &Contexte,
    int: &Interieur,
    pr: &Provider,
    r: &Reglage,
    pieces: &[Piece],
    envoi: Option<Trace>,
    avant: Option<&str>,
) -> String {
    let (fw, fh) = pr.format;
    // `leading` Typst = espace entre lignes ; `line-height` CSS = distance entre lignes
    // de base. Les deux ne coïncident qu'une fois la boîte de ligne ramenée à 1em par
    // top-edge/bottom-edge — sans quoi l'interligne dépend de la police choisie.
    let lead = INTERLIGNE - 1.0;
    let folio = format!(
        r#"context align(center, text(size: {}pt, counter(page).display()))"#,
        int.folio
    );

    // Les zones sont déjà validées par `decoupe` : le découpage n'a qu'à les suivre.
    let lim = pieces
        .iter()
        .take_while(|p| matches!(p.sorte, Sorte::Liminaire))
        .count();
    let (liminaires_manuscrit, reste) = pieces.split_at(lim);
    let corps = reste
        .iter()
        .take_while(|p| !matches!(p.sorte, Sorte::Annexe))
        .count();
    let (corps, annexes) = reste.split_at(corps);

    let mut s = String::new();
    s.push_str(&format!(
        r#"// Intérieur — {} ({})
#set document(title: "{}", author: "{}")
#set page(
  width: {fw}mm, height: {fh}mm,
  margin: (top: {}mm, bottom: {}mm, inside: {}mm, outside: {}mm),
  footer: none,{fg}
)
#set text(font: "{}", size: {}pt, lang: "fr", hyphenate: true,
          top-edge: 0.75em, bottom-edge: -0.25em,
          costs: (orphan: 100%, widow: 100%))
#set par(justify: true, leading: {lead}em, spacing: {lead}em, first-line-indent: 1.2em)

// Le blanc de respiration : `n` lignes sautées, sans marque. Faible au sens de Typst,
// donc supprimé à une frontière de page — le registre passe avant la coupure.
//
// La hauteur est exacte, pas approchée : `top-edge` et `bottom-edge` ci-dessus posent
// la ligne à 1em pile, l'avance d'une ligne à la suivante vaut donc 1em + leading. Le
// blanc doit en plus couvrir l'espacement de paragraphe qu'il remplace — Typst fusionne
// deux espacements faibles en gardant le plus grand —, d'où le terme supplémentaire.
// À n = 1 : 1em + 2·leading, la valeur relevée sur PDF le 22/08.
#let blanc(n) = v(n * 1em + (n + 1) * {lead}em, weak: true)

"#,
        // Ces trois-là sont cités, non composés : la ligne de commentaire et la chaîne
        // de `#set document` demandent l'échappement de chaîne, pas celui du markup.
        echappe_chaine(&ctx.livre.titre),
        pr.cle,
        echappe_chaine(&ctx.livre.titre),
        echappe_chaine(&ctx.livre.auteur),
        pr.marge_haut,
        pr.marge_bas,
        r.gouttiere,
        pr.exterieur,
        // La police est validée en amont par `Interieur::verifie` : pas d'échappement.
        int.police,
        int.corps,
        fg = foreground(envoi, fw),
    ));

    // La page insérée vient avant tout ce que `liminaires` écrit : c'est la page 1 du
    // fichier, celle qu'un lecteur voit en ouvrant.
    if let Some(a) = avant {
        s.push_str(a);
    }

    s.push_str(&liminaires(ctx, int, liminaires_manuscrit));

    // — Corps, folio rétabli. La numérotation court depuis le faux-titre, seul son
    //   affichage était supprimé : le premier chapitre s'ouvre donc en page 5, ou en 7
    //   quand le livre porte une dédicace. —
    s.push_str(&format!("#set page(footer: {folio})\n"));

    // `#page(…)[…]` rompt le flux de lui-même, avant et après : après une page de
    // partie, le `#pagebreak()` d'ouverture du chapitre suivant ferait une page blanche
    // de plus. Le compte de pages est le seul juge de ce détail.
    let mut apres_page = false;
    for (i, p) in corps.iter().enumerate() {
        match &p.sorte {
            Sorte::Partie(r) => {
                // Une ouverture de partie est une belle page. Le verso blanc, lui, est
                // acquis par le second `#page` — mais le recto ne l'est pas : au milieu
                // du corps, la parité dépend de la longueur du chapitre précédent, donc
                // d'un texte que l'auteur retouche. Sans ce saut, trois paragraphes
                // ajoutés au chapitre d'avant retournent le dispositif, et cela ne se
                // découvre qu'après tirage.
                //
                // Le saut n'est pas un `pagebreak(to: "odd")` : la page qu'il insère
                // hérite du folio du corps, et une page entièrement vide portant son
                // numéro au milieu du livre se remarque — aucune édition courante ne le
                // fait. La blanche est donc posée ici, sans folio, en regardant la
                // parité de la page où le flux se trouve : la partie ouvre la suivante,
                // donc c'est une page **impaire** en cours qui appelle une blanche.
                //
                // En tête de corps, rien à caler : les liminaires viennent de rendre la
                // main sur une belle page vierge. Le test y verrait une page impaire
                // « en cours » et poserait sa blanche au recto — l'inverse du but.
                if i > 0 {
                    s.push_str("#context if calc.odd(here().page()) { page(footer: none)[] }\n");
                }
                s.push_str(&format!(
                    "#page(footer: none)[\n{}#v(22mm)\n\
                     #align(center, text(size: {}pt)[{r}])\n",
                    repere(p),
                    int.numero
                ));
                s.push_str(&titre_sous_numero(&p.titre, int.titre_section));
                s.push_str("]\n#page(footer: none)[]\n");
                apres_page = true;
            }
            Sorte::Chapitre(numero) => {
                // Le premier chapitre suit le dernier saut de page des liminaires : ne
                // pas en ajouter un.
                if i > 0 && !apres_page {
                    s.push_str("#pagebreak()\n");
                }
                s.push_str(&repere(p));
                s.push_str(&format!(
                    "#v(22mm)\n#align(center, text(size: {}pt)[{numero}])\n",
                    int.numero
                ));
                s.push_str(&titre_sous_numero(&p.titre, int.titre_section));
                s.push_str("#v(11mm)\n");
                s.push_str(&blocs_typst(&p.blocs));
                apres_page = false;
            }
            // `decoupe` garantit les zones : ni liminaire ni annexe n'entre dans le corps.
            Sorte::Liminaire | Sorte::Annexe => unreachable!("zone validée par decoupe"),
        }
    }

    // Les annexes rejoignent les liminaires hors du folio : il appartient au corps.
    if !annexes.is_empty() {
        if !apres_page {
            s.push_str("#pagebreak()\n");
        }
        s.push_str("#set page(footer: none)\n");
        for (i, p) in annexes.iter().enumerate() {
            if i > 0 {
                s.push_str("#pagebreak()\n");
            }
            s.push_str(&repere(p));
            s.push_str(&ouverture_piece(&p.titre, int.ouverture_piece));
            s.push_str(&blocs_typst(&p.blocs));
        }
    }

    // La table en fin ferme le volume, annexes comprises : c'est la dernière chose du
    // livre. Elle rejoint la zone hors folio que les annexes occupent déjà — et quand il
    // n'y a pas d'annexe, c'est ici que cette zone s'ouvre, dans l'ordre qu'emploie le
    // bloc ci-dessus : le saut de page d'abord, le `set page` ensuite.
    if int.table == Table::EnFin {
        if annexes.is_empty() {
            if !apres_page {
                s.push_str("#pagebreak()\n");
            }
            s.push_str("#set page(footer: none)\n");
        }
        s.push_str(&table_matieres(int));
    }

    // Page blanche de fin, sans folio — même dispositif que la blanche des liminaires.
    if r.blanche {
        s.push_str("\n#page(footer: none)[]\n");
    }
    s.push_str(&format!("\n{MARQUEUR}\n"));
    s
}

/// Ce que l'envoi ajoute à `#set page` : un `foreground` conditionné au numéro de page.
///
/// **`foreground` et non le flux.** Un `#place` dans le flux ne pouvait déjà pas créer
/// de page ; il fallait en revanche l'écrire là où la page visée se compose, ce qui
/// enfermait l'envoi sur la page de titre. Le `foreground`, lui, se pose une fois au
/// préambule et vise n'importe quelle page — un `#set page(…)` au milieu du document
/// ouvrirait une page, d'où le préambule et lui seul.
///
/// Il survit au `#set page(footer: …)` qui ouvre le corps, les `set` de Typst
/// fusionnant champ à champ, et aux `#page(…)[…]` des pages de partie. Ses pourcentages
/// se résolvent sur la **page entière, marges comprises** : c'est ce qui les met en
/// correspondance 1:1 avec le canevas de l'interface, qui montre la page entière.
///
/// `counter(page)` n'est jamais remis à zéro dans l'intérieur — seul son affichage est
/// masqué jusqu'au corps —, si bien que la condition porte bien sur la n-ième page du
/// fichier, celle que la vignette montre.
fn foreground(envoi: Option<Trace>, largeur_mm: f64) -> String {
    let Some(t) = envoi else {
        return String::new();
    };
    let p = t.place;
    let quoi = match t.quoi {
        Quoi::Texte { police, texte } => format!(
            r#"box(width: {taille}%)[
        #set par(justify: false, first-line-indent: 0pt, leading: 0.9em)
        #text(font: "{police}", size: {corps:.3}mm, hyphenate: false)[{mot}]
      ]"#,
            taille = p.taille * 100.0,
            // La main est validée en amont par `Envois::verifie` : pas d'échappement.
            mot = echappe(texte).replace('\n', r" \ "),
            corps = p.taille * largeur_mm * CORPS_SUR_LARGEUR,
        ),
        // Le nom du fichier est fabriqué par `envoi::nom_image` : assaini, il ne porte
        // ni guillemet qui refermerait la chaîne, ni séparateur qui la ferait sortir du
        // répertoire où l'image vient d'être écrite.
        //
        // Aucune borne de hauteur, contrairement à la v3 : elle protégeait d'un envoi
        // qui recouvrirait le titre, or le canevas montre désormais ce recouvrement, et
        // le brider corrigerait l'auteur d'une faute qu'il voit.
        Quoi::Image { fichier } => format!(r#"image("{fichier}", width: {}%)"#, p.taille * 100.0),
    };
    format!(
        r#"
  foreground: context {{
    if counter(page).get().first() == {page} {{
      place(center + horizon, dx: {dx}%, dy: {dy}%, rotate({angle}deg, {quoi}))
    }}
  }},"#,
        page = p.page,
        dx = (p.x - 0.5) * 100.0,
        dy = (p.y - 0.5) * 100.0,
        angle = p.angle,
    )
}

/// La source d'un envoi rendu **seul**, sur fond transparent, à hauteur automatique.
///
/// C'est ce que le canevas de placement manipule. Le rendre par Typst plutôt que de
/// l'imiter en CSS fait que ce qu'on déplace **est** ce qui s'imprimera — même police,
/// même corps, même coupure de lignes. La page en fond ne bouge pas, un `foreground` ne
/// réordonnant rien : glisser, redimensionner et incliner ne sont plus alors que des
/// `transform`, et Typst n'est rappelé que quand le mot ou la main changent.
///
/// `fill: none` donne le fond transparent, `height: auto` laisse la hauteur suivre le
/// texte. La largeur est celle que l'objet occupera sur la page : c'est elle qui décide
/// des coupures de lignes, et la rendre à une autre largeur donnerait un objet dont le
/// rapport ne serait pas celui du rendu.
pub fn source_objet(t: &Trace, largeur_mm: f64) -> String {
    let quoi = match &t.quoi {
        Quoi::Texte { police, texte } => format!(
            r#"#set par(justify: false, first-line-indent: 0pt, leading: 0.9em)
#set text(font: "{police}", size: {corps:.3}mm, hyphenate: false, lang: "fr")
{mot}
"#,
            corps = largeur_mm * CORPS_SUR_LARGEUR,
            // La main est validée en amont par `Envois::verifie` : pas d'échappement.
            mot = echappe(texte).replace('\n', r" \ "),
        ),
        Quoi::Image { fichier } => format!("#image(\"{fichier}\", width: 100%)\n"),
    };
    format!("#set page(width: {largeur_mm}mm, height: auto, margin: 0pt, fill: none)\n{quoi}")
}

/// Source Typst de l'intérieur du livre, tel qu'il part à l'impression.
pub fn source(
    livre: &Livre,
    int: &Interieur,
    pr: &Provider,
    r: &Reglage,
    pieces: &[Piece],
    envoi: Option<Trace>,
) -> String {
    // Ce qui part à l'impression connaît son imprimeur : c'est le seul endroit où il
    // entre dans le livre, et il ne vient pas du livre mais du gabarit visé.
    let ctx = Contexte {
        livre,
        imprimeur: Some(&pr.pod_nom),
    };
    assemble(&ctx, int, pr, r, pieces, envoi, None)
}

/// L'intérieur du livre précédé de sa couverture, **sans imposition**.
///
/// La gouttière revient à la marge extérieure et la blanche de parité disparaît : ce
/// ne sont pas des réglages qu'on offre, c'est ce que veut dire « sans imposition ».
/// Les deux n'ont de sens qu'une fois le livre relié.
///
/// Aucun envoi : l'envoi autographe est une affaire de tirage papier, et il n'a pas de
/// dédicataire ici.
pub fn source_ebook(
    livre: &Livre,
    int: &Interieur,
    pr: &Provider,
    pieces: &[Piece],
    couverture: &str,
) -> String {
    let r = Reglage {
        gouttiere: pr.exterieur,
        blanche: false,
    };
    // Aucun imprimeur : le format vient d'un gabarit, mais rien n'est imprimé.
    let ctx = Contexte {
        livre,
        imprimeur: None,
    };
    assemble(&ctx, int, pr, &r, pieces, None, Some(couverture))
}

/// Les pages liminaires : faux-titre, blanche, page de titre, copyright, et — quand le
/// livre en porte une — la dédicace et sa blanche, puis les pièces liminaires du
/// manuscrit.
///
/// Toutes sans folio, et sans avoir à le dire : `footer: none`, posé par l'entête que
/// `source` écrit, court jusqu'au `#set page(footer: …)` qui ouvre le corps.
fn liminaires(ctx: &Contexte, int: &Interieur, pieces: &[Piece]) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"#v(42mm)
#align(center, text(size: {}pt, tracking: 0.12em)[{}])
#pagebreak()
#pagebreak()

#v(30mm)
#align(center, text(size: {}pt, tracking: 0.1em)[{}])
#v(14mm)
#align(center, text(size: {}pt, tracking: 0.06em)[{}])
#v(10mm)
#align(center, emph(text(size: {}pt)[{}]))
"#,
        int.faux_titre,
        majuscules(&ctx.livre.titre),
        int.page_titre_auteur,
        majuscules(&ctx.livre.auteur),
        int.page_titre_titre,
        majuscules(&ctx.livre.titre_page(ctx.imprimeur).replace('\n', "\u{1}"))
            .replace('\u{1}', r" \ "),
        int.page_titre_genre,
        echappe(&ctx.livre.genre),
    ));

    s.push_str("#pagebreak()\n\n");

    // Le pavé de copyright est calé en bas de la justification. La chaîne Python le
    // posait à 143 mm du haut du corps — une valeur juste pour le poche Lulu et
    // arbitraire ailleurs ; le bas de la justification est la même intention, exprimée
    // indépendamment du format.
    s.push_str(&format!(
        r#"#place(bottom + center, block(width: 100%)[
  #set par(leading: 0.5em, spacing: 0.5em, first-line-indent: 0pt, justify: false)
  #align(center, text(size: {}pt)[{}])
])
#pagebreak()

"#,
        int.copyright,
        echappe(&ctx.livre.copyright(ctx.imprimeur)).replace('\n', r" \ ")
    ));

    // La dédicace prend une belle page, son verso reste blanc — deux `#pagebreak()`
    // d'affilée, le dispositif de la blanche du faux-titre. Le corps s'ouvre donc en
    // page 7 au lieu de 5, et le dos en tient compte de lui-même puisqu'il découle de
    // la pagination mesurée, jamais d'une saisie.
    if let Some(d) = ctx.livre.dedicace(ctx.imprimeur) {
        s.push_str(&format!(
            r#"#v(48mm)
#align(right, emph(text(size: {}pt)[{}]))
#pagebreak()
#pagebreak()

"#,
            int.dedicace,
            echappe(&d).replace('\n', r" \ ")
        ));
    }

    // La table en tête rejoint la série des liminaires : après le copyright et la
    // dédicace, **avant** la préface. Le lecteur trouve le plan du livre sans traverser
    // un texte, et la table annonce la préface elle-même.
    //
    // `footer: none` court encore ici : la table est hors folio, comme tout ce qui la
    // précède.
    if int.table == Table::EnTete {
        s.push_str(&table_matieres(int));
        // Ce qui suit ouvre en belle page — la préface, ou le corps. Même dispositif
        // qu'après une pièce liminaire, et pour la même raison : la longueur de la table
        // dépend du nombre de pièces, donc d'un manuscrit qu'on retouche.
        s.push_str("#pagebreak(to: \"odd\", weak: true)\n\n");
    }

    // Les pièces liminaires du manuscrit ferment la série : `footer: none` court encore,
    // le folio ne sera rétabli qu'au premier chapitre.
    for p in pieces {
        s.push_str(&repere(p));
        s.push_str(&ouverture_piece(&p.titre, int.ouverture_piece));
        s.push_str(&blocs_typst(&p.blocs));
        // Ce qui suit une pièce liminaire ouvre en belle page — le corps, ou la pièce
        // suivante. Sa longueur, elle, dépend d'un texte que l'auteur retouche : une
        // page de plus ou de moins à la préface renversait la parité de tout ce qui
        // vient après, et le corps s'ouvrait au verso sans que rien ne le dise.
        //
        // Ici, et à la différence de la page de partie, le saut de parité est le bon
        // outil : `footer: none` court encore, donc la blanche qu'il insère n'est pas
        // foliotée — c'est la seule objection que le corps lui oppose. Le compter à la
        // main sur `here().page()` serait au contraire faux : en fin de page, un
        // élément de taille nulle est déjà rendu sur la page suivante, et le calage
        // poserait sa blanche au recto — relevé sur PDF, une préface de deux pages.
        s.push_str("#pagebreak(to: \"odd\", weak: true)\n\n");
    }

    s
}

/// Majuscules typographiques : `upper()` de Typst plutôt qu'une bascule en Rust, pour
/// que la casse suive la langue du document (le CSS faisait `text-transform`).
fn majuscules(s: &str) -> String {
    format!("#upper[{}]", echappe(s))
}

/// L'ouverture d'une pièce à texte — préface, postface.
///
/// Le mot occupe la ligne du numéro, mais composé comme un **titre** de chapitre : ce
/// sont la casse et l'espacement qui font le titre, la taille du numéro étant celle
/// d'un chiffre isolé. Le blanc de 14,5 mm est la somme des deux blancs du gabarit
/// (3,5 + 11) : le texte s'ouvre à la même hauteur que celui d'un chapitre.
///
/// Sa taille se règle à part de `titre_section` bien qu'elles vaillent la même chose
/// par défaut : ici le mot est seul sur sa ligne, là il vient sous un numéro.
fn ouverture_piece(titre: &str, pt: f64) -> String {
    format!(
        "#v(22mm)\n#align(center, text(size: {pt}pt, tracking: 0.14em)[{}])\n#v(14.5mm)\n",
        majuscules(titre)
    )
}

/// Le titre que la table porte, dans les deux positions.
///
/// Un seul libellé, et non « Sommaire » en tête : rien à expliquer dans l'interface, et
/// c'est le mot que tout lecteur reconnaît. Décision de produit du 29/08.
const TITRE_TABLE: &str = "Table des matières";

/// L'étiquette que porte chaque repère de table, telle qu'une requête Typst la nomme.
///
/// Publique parce que la table la lira — `context query(<ozalid-tdm>)` — et qu'un nom
/// recopié à deux endroits est un nom qui divergera.
pub const TDM: &str = "ozalid-tdm";

/// Le repère qu'une pièce laisse à l'ouverture de sa page, pour la table des matières.
///
/// **Il ne s'affiche pas et n'occupe aucune place** : un `metadata` n'est pas mis en
/// page, il est seulement situé. C'est ce qui permet de le poser dans tous les livres,
/// table allumée ou non, et de prouver par le témoin qu'il ne coûte rien — une preuve
/// impossible si la pose dépendait du réglage, puisque l'allumer changerait alors deux
/// choses à la fois.
///
/// Trois champs, et non un libellé prémâché : le rang indente, le numéro et le titre
/// sont ce que la page d'ouverture imprime. Composer la ligne ici enfermerait la mise
/// en forme dans le Rust, alors qu'elle appartient à la table.
///
/// Les valeurs sont **citées, non composées** : `echappe_chaine`, jamais `echappe`.
fn repere(p: &Piece) -> String {
    let (rang, numero) = match &p.sorte {
        // Une partie tient le premier rang ; tout le reste est indenté sous elle.
        Sorte::Partie(romain) => (1, romain.clone()),
        Sorte::Chapitre(n) => (2, n.to_string()),
        Sorte::Liminaire | Sorte::Annexe => (2, String::new()),
    };
    format!(
        "#metadata((rang: {rang}, numero: \"{}\", titre: \"{}\"))<{TDM}>\n",
        echappe_chaine(&numero),
        echappe_chaine(&p.titre)
    )
}

/// La table des matières, composée par Typst depuis les repères que chaque pièce a
/// laissés à l'ouverture de sa page.
///
/// **Typst résout seul l'auto-référence** : la table occupe des pages, et les folios
/// qu'elle affiche en tiennent compte, en une seule invocation. Relevé par composition
/// le 29/08 sur une table de deux pages — les folios sortent consécutifs à partir de la
/// page qui suit la table, pas de celle qui l'aurait suivie sans elle. Les deux voies
/// écartées sont dans la spec § 2.3 : `outline()` natif, que l'intérieur ne peut pas
/// employer faute d'un seul `heading`, et deux passes côté Rust, qui devraient itérer
/// jusqu'au point fixe comme `converge` le fait pour la gouttière.
///
/// **La table ne porte pas l'étiquette `<ozalid-tdm>`** : elle se listerait elle-même.
///
/// Elle s'ouvre en belle page. La blanche qui la suit, quand elle finit sur une impaire,
/// appartient à l'appelant — en tête c'est le saut de parité qui ouvre la pièce
/// suivante, en fin c'est la blanche de parité du livre. Un saut de sortie posé ici
/// ajouterait une page en fin de volume que rien n'occuperait.
///
/// L'indentation du second rang ne paraît que si le livre porte une partie : un roman
/// sans parties verrait sinon toutes ses lignes décalées sous un rang qui n'existe pas.
fn table_matieres(int: &Interieur) -> String {
    let mut s = String::from("#pagebreak(to: \"odd\", weak: true)\n");
    // La table s'ouvre comme une préface : c'est une pièce du livre, et le mot occupe
    // la ligne du numéro.
    s.push_str(&ouverture_piece(TITRE_TABLE, int.ouverture_piece));
    // Le `set par` local défait la justification et l'alinéa du corps : une ligne de
    // table justifiée écarterait ses points de conduite jusqu'à la marge.
    s.push_str(&format!(
        r#"#context {{
  let entrees = query(<{TDM}>)
  let parties = entrees.any(e => e.value.rang == 1)
  set par(justify: false, first-line-indent: 0pt, leading: 0.6em, spacing: 0.6em)
  set text(size: {pt}pt)
  for e in entrees {{
    let v = e.value
    let libelle = if v.numero == "" {{ v.titre }} else if v.titre == "" {{ v.numero }} else {{ v.numero + " — " + v.titre }}
    block(above: if v.rang == 1 {{ 1.2em }} else {{ 0.6em }})[
      #h(if v.rang == 1 or not parties {{ 0mm }} else {{ 5mm }})#if v.rang == 1 {{ upper(libelle) }} else {{ libelle }}#box(width: 1fr, repeat[#h(0.3em).#h(0.3em)])#e.location().page()
    ]
  }}
}}
"#,
        pt = int.entree_table
    ));
    s
}

/// Le titre sous le numéro d'une partie ou d'un chapitre — même casse, même espacement
/// que l'un ou l'autre, puisque c'est le même gabarit qui les compose. Absent si la
/// pièce n'a pas de titre : c'est le cas admis par le format (`## 7`, `## Partie I`).
fn titre_sous_numero(titre: &str, pt: f64) -> String {
    if titre.is_empty() {
        return String::new();
    }
    format!(
        "#v(3.5mm)\n#align(center, text(size: {pt}pt, tracking: 0.14em)[{}])\n",
        majuscules(titre)
    )
}

/// Les blocs d'une pièce, composés. Partagé par les chapitres et les pièces à texte :
/// une préface se lit dans la même page qu'un chapitre.
fn blocs_typst(blocs: &[Bloc]) -> String {
    let mut s = String::new();
    for b in blocs {
        match b {
            Bloc::Paragraphe(p) => {
                s.push_str(&inline(p));
                s.push_str("\n\n");
            }
            // Le blanc est en em, non en mm : il suit le corps de l'intérieur comme
            // l'interligne, là où l'épreuve, qui n'a qu'un format, se règle en mm.
            // Il s'ajoute à l'espace de paragraphe, de part et d'autre — une rupture
            // se voit d'un coup d'œil sur la page, sans la trouer.
            //
            // Le paragraphe qui suit garde son alinéa, comme après n'importe quel
            // blanc : relevé sur la page composée, pas déduit. La marque le rend
            // sans conséquence — c'est elle qui dit la coupure, pas le retrait.
            Bloc::Scene => s.push_str(&format!("#v(1em)\n#align(center)[{SCENE}]\n#v(1em)\n\n")),
            // Le blanc n'a pas de marque, donc rien à centrer : il est tout entier
            // dans l'espace. Sa hauteur est définie une fois au préambule, là où
            // l'interligne est connue — une ligne de texte laissée vide.
            Bloc::Blanc(n) => s.push_str(&format!("#blanc({n})\n\n")),
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::provider;
    use crate::envoi::Place;
    use crate::typst::Typst;
    use std::cell::RefCell;
    use std::path::Path;

    fn livre() -> Livre {
        Livre {
            isbn: String::new(),
            depot_legal: String::new(),
            titre: "Les Heures creuses".into(),
            titre_page: "Les Heures\ncreuses".into(),
            auteur: "Ivan Pjig".into(),
            genre: "roman".into(),
            editeur: "Editeur".into(),
            collection: "Collection".into(),
            monogramme: "Monogramme".into(),
            copyright: "© Ivan Pjig, 2026.\nTous droits réservés.".into(),
            prix: "Prix".into(),
            mention: "Mention".into(),
            dedicace: String::new(),
            chapitres: None,
        }
    }

    fn chapitres() -> Vec<Piece> {
        vec![Piece {
            sorte: Sorte::Chapitre(1),
            titre: "Un".into(),
            blocs: vec![Bloc::Paragraphe("Texte.".into())],
        }]
    }

    fn pieces_avec_blanc() -> Vec<Piece> {
        vec![Piece {
            sorte: Sorte::Chapitre(1),
            titre: "Un".into(),
            blocs: vec![
                Bloc::Paragraphe("Avant.".into()),
                Bloc::Blanc(1),
                Bloc::Paragraphe("Après.".into()),
            ],
        }]
    }

    /// Le blanc est un espace, pas un signe : la source ne doit porter aucune marque
    /// pour lui. C'est toute la différence avec la rupture de scène, et elle se vérifie
    /// ici plutôt qu'après tirage.
    #[test]
    fn le_blanc_de_respiration_ne_compose_aucune_marque() {
        let s = blocs_typst(&[
            Bloc::Paragraphe("Avant.".into()),
            Bloc::Blanc(1),
            Bloc::Paragraphe("Après.".into()),
        ]);
        assert!(s.contains("#blanc"), "{s}");
        assert!(!s.contains(SCENE), "{s}");
    }

    /// Le blanc est faible au sens de Typst : il disparaît à une frontière de page.
    /// C'est ce qui protège le registre — sans `weak`, la page suivante s'ouvrirait sur
    /// un trou et ses lignes ne seraient plus en regard de celles d'en face.
    ///
    /// Sa hauteur vaut `n` lignes, relevé sur PDF : Typst fusionne deux espacements
    /// faibles adjacents en gardant le plus grand, d'où le terme qui couvre l'espacement
    /// de paragraphe remplacé. Mesuré à 10 pt, `leading` et `spacing` à 0,65em, en
    /// lisant `here().position().y` après le blanc : n = 1 → 57 pt, n = 2 → 73,5 pt,
    /// n = 3 → 90 pt, soit 16,5 pt — une avance de ligne — par ligne demandée, et 90 pt
    /// aussi pour trois vraies lignes de texte à la place du blanc. À n = 1, la valeur
    /// est celle de l'ancienne formule `1em + lead * 2` : aucun manuscrit ne bouge.
    #[test]
    fn le_blanc_de_respiration_est_un_espace_faible() {
        let pr = provider("bod").unwrap();
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(
            &livre(),
            &Interieur::default(),
            pr,
            &r,
            &pieces_avec_blanc(),
            None,
        );
        assert!(s.contains("#let blanc(n) = v("), "{s}");
        assert!(s.contains("weak: true"), "{s}");
    }

    /// Un blanc de plusieurs lignes se compose en **un** espacement, jamais en plusieurs
    /// marques à la file : Typst fusionne deux espacements faibles adjacents en gardant
    /// le plus grand, et trois `#blanc(1)` n'auraient sauté qu'une ligne.
    #[test]
    fn un_blanc_de_trois_lignes_ne_compose_qu_un_espacement() {
        let s = blocs_typst(&[
            Bloc::Paragraphe("Avant.".into()),
            Bloc::Blanc(3),
            Bloc::Paragraphe("Après.".into()),
        ]);
        assert!(s.contains("#blanc(3)"), "{s}");
        assert_eq!(s.matches("#blanc(").count(), 1, "{s}");
    }

    /// Une composition déjà stable ne doit pas être recomposée : une reprise inutile
    /// coûte une passe de mise en page sur tout le livre.
    #[test]
    fn une_composition_stable_converge_du_premier_coup() {
        // Un gabarit à tranche unique : chez Lulu, cinq tranches se succèdent, et 272
        // pages ne tombent pas dans celle où la convergence commence — ce serait alors la
        // recomposition qu'on mesurerait, pas la stabilité.
        let pr = provider("bod").unwrap();
        let appels = RefCell::new(0);
        let r = converge(pr, |_| {
            *appels.borrow_mut() += 1;
            Ok(272)
        })
        .unwrap();
        assert_eq!(r.pages, 272);
        assert_eq!(r.gouttiere, 20.0);
        assert!(!r.blanche);
        assert_eq!(*appels.borrow(), 1);
    }

    /// Un compte impair est corrigé par la blanche de fin, et le compte retenu est
    /// celui de la composition **avec** la blanche — pas celui d'avant.
    #[test]
    fn un_compte_impair_ajoute_la_blanche_et_repart_du_nouveau_compte() {
        // Tranche unique, pour la même raison : c'est la blanche qu'on compte ici, pas un
        // changement de gouttière.
        let pr = provider("bod").unwrap();
        let n = RefCell::new(0);
        let r = converge(pr, |reglage| {
            *n.borrow_mut() += 1;
            Ok(if reglage.blanche { 272 } else { 271 })
        })
        .unwrap();
        assert!(r.blanche);
        assert_eq!(r.pages, 272);
        assert_eq!(*n.borrow(), 2);
    }

    /// Le cas qui justifie la boucle : la gouttière dépend de la pagination, et la
    /// changer déplace la pagination. Le réglage retenu doit être cohérent avec le
    /// compte final, pas avec l'hypothèse de départ.
    #[test]
    fn un_changement_de_tranche_recompose_avec_la_bonne_gouttiere() {
        let pr = provider("kdp-6x9").unwrap();
        let r = converge(pr, |reglage| {
            // Avec la gouttière étroite le livre tient en 700 pages ; l'élargir le
            // fait passer dans la tranche suivante, qui impose l'autre gouttière.
            Ok(if reglage.gouttiere < 20.0 { 702 } else { 720 })
        })
        .unwrap();
        assert_eq!(r.gouttiere, 22.23);
        assert_eq!(r.pages, 720);
    }

    /// Hors tranche connue, la convergence s'arrête sur le message du gabarit : elle
    /// ne doit pas boucler ni retenir une gouttière inventée.
    #[test]
    fn une_pagination_hors_tranche_interrompt_la_convergence() {
        let pr = provider("lulu").unwrap();
        // Vingt pages : sous les 32 que le broché de Lulu admet, donc sous sa première
        // tranche de gouttière.
        let err = converge(pr, |_| Ok(20)).unwrap_err();
        assert!(err.contains("20 pages"), "{err}");
    }

    /// Une oscillation doit finir par échouer plutôt que tourner sans fin — sans quoi
    /// l'app se figerait sur un manuscrit pathologique.
    #[test]
    fn une_oscillation_est_bornee_et_signalee() {
        let pr = provider("lulu").unwrap();
        let tour = RefCell::new(0u32);
        let err = converge(pr, |_| {
            let mut t = tour.borrow_mut();
            *t += 1;
            Ok(if (*t).is_multiple_of(2) { 271 } else { 273 })
        })
        .unwrap_err();
        assert!(err.contains("ne converge pas"), "{err}");
    }

    #[test]
    fn la_source_porte_le_gabarit_de_l_imprimeur_et_le_marqueur() {
        let pr = provider("bod").unwrap();
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(&livre(), &Interieur::default(), pr, &r, &[], None);
        assert!(s.contains("width: 135mm, height: 215mm"));
        assert!(s.contains("inside: 20mm"), "gouttière absente");
        assert!(s.contains("outside: 15mm"));
        assert!(s.contains("costs: (orphan: 100%, widow: 100%)"), "veuves");
        assert!(s.trim_end().ends_with(MARQUEUR), "marqueur de pagination");
    }

    /// Le pavé de copyright de la page 4 cite l'imprimeur, et l'imprimeur vient du
    /// gabarit visé : c'est la même mécanique que le dos, où le chiffre ne passe jamais
    /// par un humain.
    #[test]
    fn le_copyright_de_l_interieur_cite_l_imprimeur_du_gabarit() {
        let mut l = livre();
        l.copyright = "Imprimé par %IMPRIMEUR%".into();
        let pr = provider("bod").unwrap();
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(&l, &Interieur::default(), pr, &r, &[], None);
        assert!(!pr.pod_nom.is_empty());
        assert!(
            s.contains(&pr.pod_nom),
            "le nom de l'imprimeur manque à la page 4"
        );
        assert!(!s.contains("%IMPRIMEUR%"), "le jeton est resté littéral");
    }

    /// L'ebook n'est pas imprimé : le jeton s'y efface. Le même livre, la même page 4,
    /// et une mention qui n'a pas de sens sur un écran.
    #[test]
    fn l_ebook_n_a_pas_d_imprimeur() {
        let mut l = livre();
        l.copyright = "Imprimé par %IMPRIMEUR%".into();
        let pr = provider("bod").unwrap();
        let s = source_ebook(&l, &Interieur::default(), pr, &[], "");
        assert!(!s.contains(&pr.pod_nom), "l'ebook a nommé un imprimeur");
        assert!(!s.contains("%IMPRIMEUR%"), "le jeton est resté littéral");
    }

    /// Le corps et l'interligne ne sont pas des faits d'imprimeur : ils étaient
    /// identiques dans les quatorze entrées de la table. Ils vivent ici désormais, et la
    /// source composée les porte quel que soit le gabarit visé — un poche et un grand
    /// format se composent au même corps.
    #[test]
    fn le_corps_et_l_interligne_ne_dependent_pas_de_l_imprimeur() {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        for cle in ["lulu", "kdp-6x9"] {
            let pr = provider(cle).unwrap();
            let s = source(&livre(), &Interieur::default(), pr, &r, &[], None);
            assert!(s.contains(&format!("size: {CORPS_PT}pt")), "{cle} : {s}");
            assert!(
                s.contains(&format!("leading: {}em", INTERLIGNE - 1.0)),
                "{cle} : {s}"
            );
        }
    }

    /// Le folio est le seul des trois réglages déménagés dont plus rien ne verrait la
    /// valeur changer : la pagination n'en dépend pas, donc le témoin le laisse passer.
    /// Sa valeur historique est donc épinglée ici, en clair — écrite avec la constante,
    /// l'assertion ne dirait plus rien.
    #[test]
    fn le_folio_garde_le_corps_de_huit_points_de_la_table() {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("bod").unwrap(),
            &r,
            &[],
            None,
        );
        assert!(
            s.contains("text(size: 8pt, counter(page).display())"),
            "{s}"
        );
    }

    /// Le livre et le manuscrit qui font paraître les onze rôles d'un coup : la dédicace
    /// pour sa page, une pièce liminaire pour son ouverture, une page de partie et un
    /// chapitre pour le numéro et le titre qu'ils partagent.
    fn livre_complet() -> Livre {
        Livre {
            dedicace: "À M., qui a tenu la lampe.".into(),
            ..livre()
        }
    }

    fn pieces_completes() -> Vec<Piece> {
        vec![
            Piece {
                sorte: Sorte::Liminaire,
                titre: "Préface".into(),
                blocs: vec![Bloc::Paragraphe("Avant.".into())],
            },
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Première".into(),
                blocs: vec![],
            },
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
        ]
    }

    /// La source complète d'un livre qui porte les onze rôles.
    fn source_complete(int: &Interieur) -> String {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        source(
            &livre_complet(),
            int,
            provider("bod").unwrap(),
            &r,
            &pieces_completes(),
            None,
        )
    }

    /// Les onze tailles deviennent des réglages **sans que la page bouge** : les défauts
    /// de `Interieur` rendent exactement les littéraux qui vivaient dans le module.
    ///
    /// C'est le témoin de la migration, et il est écrit en clair : reprendre les
    /// constantes ferait une assertion qui ne dit rien, et une valeur déplacée par
    /// mégarde passerait sans un mot — en déplaçant la pagination, donc le dos.
    #[test]
    fn les_defauts_reproduisent_les_tailles_codees_en_dur() {
        let s = source_complete(&Interieur::default());
        for attendu in [
            r#"size: 9.5pt, lang: "fr""#,
            "text(size: 11pt, tracking: 0.12em)",
            "text(size: 10.5pt, tracking: 0.1em)",
            "text(size: 15pt, tracking: 0.06em)",
            "emph(text(size: 10pt)",
            "text(size: 8pt)[©",
            "emph(text(size: 9.5pt)[À M.",
            "text(size: 13pt)[I]",
            "text(size: 10pt, tracking: 0.14em)",
            "text(size: 8pt, counter(page).display())",
        ] {
            assert!(s.contains(attendu), "taille déplacée : {attendu}\n{s}");
        }
    }

    /// Chaque rôle typographique prend la taille qu'on lui règle, et lui seule.
    ///
    /// Onze valeurs distinctes, parce que trois rôles partagent aujourd'hui la même :
    /// le genre, le titre sous le numéro et l'ouverture de pièce valent tous 10 pt par
    /// défaut, et un test écrit sur les défauts ne saurait pas dire lequel a bougé.
    ///
    /// Les deux mutualisations voulues sont vérifiées par le compte : le numéro sert à
    /// la page de partie **et** au chapitre, le titre de section aux deux également.
    ///
    /// La douzième taille — l'entrée de table — ne paraît dans aucune source tant que le
    /// réglage est absent : elle est couverte par
    /// `la_taille_d_entree_regle_les_lignes_de_la_table`.
    #[test]
    fn chaque_role_typographique_prend_sa_taille() {
        let int = Interieur {
            corps: 11.25,
            faux_titre: 12.25,
            page_titre_auteur: 13.25,
            page_titre_titre: 14.25,
            page_titre_genre: 15.25,
            copyright: 16.25,
            dedicace: 17.25,
            numero: 18.25,
            titre_section: 19.25,
            ouverture_piece: 20.25,
            folio: 21.25,
            ..Interieur::default()
        };
        let s = source_complete(&int);
        for attendu in [
            r#"size: 11.25pt, lang: "fr""#,
            "text(size: 12.25pt, tracking: 0.12em)",
            "text(size: 13.25pt, tracking: 0.1em)",
            "text(size: 14.25pt, tracking: 0.06em)",
            "emph(text(size: 15.25pt)",
            "text(size: 16.25pt)[©",
            "emph(text(size: 17.25pt)[À M.",
            "text(size: 20.25pt, tracking: 0.14em)",
            "text(size: 21.25pt, counter(page).display())",
        ] {
            assert!(s.contains(attendu), "taille ignorée : {attendu}\n{s}");
        }
        assert_eq!(
            s.matches("text(size: 18.25pt)").count(),
            2,
            "le numéro vaut pour la partie et pour le chapitre\n{s}"
        );
        assert_eq!(
            s.matches("text(size: 19.25pt, tracking: 0.14em)").count(),
            2,
            "le titre de section vaut pour la partie et pour le chapitre\n{s}"
        );
    }

    /// Une taille hors bornes est refusée à la saisie, et nommée.
    ///
    /// Sans ce contrôle, un 0 tapé dans un champ ne produit pas d'erreur : Typst compose
    /// un texte de corps nul, le PDF sort blanc, et la pagination qui en découle donne
    /// un dos faux. Le champ du formulaire pose les mêmes bornes, mais un `.ozalid`
    /// écrit à la main ne passe pas par le champ.
    #[test]
    fn une_taille_hors_bornes_est_refusee_et_nommee() {
        for (quoi, mauvais) in [
            (
                "corps",
                Interieur {
                    corps: 0.0,
                    ..Interieur::default()
                },
            ),
            (
                "copyright",
                Interieur {
                    copyright: MIN_PT - 0.1,
                    ..Interieur::default()
                },
            ),
            (
                "folio",
                Interieur {
                    folio: MAX_PT + 0.1,
                    ..Interieur::default()
                },
            ),
            (
                "dédicace",
                Interieur {
                    dedicace: f64::NAN,
                    ..Interieur::default()
                },
            ),
        ] {
            let err = mauvais.verifie().unwrap_err();
            assert!(err.contains(quoi), "l'erreur doit nommer le rôle : {err}");
        }
        for bord in [MIN_PT, MAX_PT] {
            let int = Interieur {
                corps: bord,
                ..Interieur::default()
            };
            assert!(int.verifie().is_ok(), "{bord} pt est admis");
        }
    }

    /// La table naît **absente**, et un `.ozalid` écrit avant ce lot la relit absente.
    ///
    /// C'est le même parti que la collection sur le dos : allumée d'office, elle
    /// ajouterait des pages à tous les livres déjà composés, donc changerait leur dos
    /// sans que personne l'ait demandé. `VERSION` n'a pas à bouger pour autant —
    /// `#[serde(default)]` porte sur la structure entière, et un projet ancien reçoit
    /// exactement le livre qu'il composait.
    #[test]
    fn la_table_nait_absente_et_un_projet_ancien_la_relit_absente() {
        assert_eq!(Interieur::default().table, Table::Absente);
        let ancien: Interieur = serde_json::from_str("{}").expect("un projet sans intérieur");
        assert_eq!(
            ancien.table,
            Table::Absente,
            "un .ozalid ancien s'allume tout seul"
        );
        assert_eq!(
            ancien.entree_table,
            Interieur::default().entree_table,
            "la douzième taille manque à un projet ancien"
        );
    }

    /// Les trois états passent la frontière dans la forme que le front envoie.
    ///
    /// Le sélecteur de l'onglet Livre pose `"en-tete"` dans la valeur de son option :
    /// une sérialisation en `"EnTete"` ferait échouer `interieur_modifier` sur un
    /// message de serde, à mi-chemin entre les deux côtés, là où rien ne se lit.
    #[test]
    fn les_trois_etats_de_la_table_se_serialisent_en_kebab() {
        for (etat, attendu) in [
            (Table::Absente, r#""absente""#),
            (Table::EnTete, r#""en-tete""#),
            (Table::EnFin, r#""en-fin""#),
        ] {
            let json = serde_json::to_string(&etat).expect("état sérialisable");
            assert_eq!(json, attendu);
            let relu: Table = serde_json::from_str(&json).expect("état relisible");
            assert_eq!(relu, etat);
        }
    }

    /// La douzième taille est bornée comme les onze autres, et l'erreur la nomme.
    ///
    /// `tailles()` est la seule liste que `verifie()` parcourt : un champ ajouté à la
    /// struct mais oublié dans la liste passerait à 0 pt sans un mot, et Typst
    /// composerait une table invisible dont la pagination donnerait un dos faux.
    #[test]
    fn la_taille_d_entree_de_table_est_bornee_comme_les_autres() {
        let mauvais = Interieur {
            entree_table: 0.0,
            ..Interieur::default()
        };
        let err = mauvais.verifie().unwrap_err();
        assert!(
            err.contains("table des matières"),
            "l'erreur doit nommer le rôle : {err}"
        );
        // Le compte de douze tailles est garanti par la signature de `tailles()` :
        // `[(&'static str, f64); 12]`. Une assertion sur la longueur ne pourrait jamais
        // échouer, et tomberait donc sous la même règle que tout test qui n'a jamais été
        // rouge : elle ne protège rien. Seul le contrôle de sa présence dans `verifie()`
        // importe, et c'est ce que la mutation du brief teste.
    }

    /// L'ebook est le livre **sans son imposition** : la gouttière revient à la marge
    /// extérieure, et il n'y a pas de blanche de parité. Les deux n'ont de sens qu'une fois
    /// le livre relié — à l'écran, l'une décale le texte une page sur deux et l'autre ajoute
    /// une page vide.
    #[test]
    fn l_ebook_compose_sans_gouttiere_ni_blanche_de_parite() {
        let pr = provider("lulu").unwrap();
        let s = source_ebook(
            &livre(),
            &Interieur::default(),
            pr,
            &chapitres(),
            "#page[couverture]\n",
        );
        assert!(
            s.contains(&format!("inside: {}mm", pr.exterieur)),
            "gouttière non ramenée à la marge extérieure : {s}"
        );
        assert!(
            !s.contains("#page(footer: none)[]"),
            "blanche de parité présente : {s}"
        );
    }

    /// La couverture est la **première** page : avant le faux-titre, donc avant tout ce que
    /// `liminaires` écrit.
    #[test]
    fn la_couverture_precede_les_liminaires() {
        let s = source_ebook(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &chapitres(),
            "#page[COUVERTURE]\n",
        );
        let couverture = s.find("COUVERTURE").expect("couverture absente");
        let faux_titre = s.find("#v(42mm)").expect("faux-titre absent");
        assert!(couverture < faux_titre, "{s}");
    }

    /// L'intérieur d'impression ne bouge pas : `source` reste ce qu'elle était, sans page
    /// insérée. C'est ce test qui dit que le refactor n'a pas fui dans le livre papier.
    #[test]
    fn l_interieur_d_impression_ne_porte_aucune_couverture() {
        let r = Reglage {
            gouttiere: 15.0,
            blanche: true,
        };
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &r,
            &chapitres(),
            None,
        );
        assert!(s.contains("inside: 15mm"), "{s}");
        assert!(s.contains("#page(footer: none)[]"), "{s}");
    }

    /// La blanche de fin doit être sans folio : un numéro sur une page vide de fin est
    /// un défaut d'impression visible.
    #[test]
    fn la_blanche_de_fin_est_sans_folio() {
        let pr = provider("lulu").unwrap();
        let sans = source(
            &livre(),
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &[],
            None,
        );
        let avec = source(
            &livre(),
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: 25.0,
                blanche: true,
            },
            &[],
            None,
        );
        assert!(!sans.contains("#page(footer: none)[]"));
        assert!(avec.contains("#page(footer: none)[]"));
    }

    /// Le titre de la page de titre garde ses sauts de ligne voulus, et rien de ce qui
    /// vient du projet ne peut ouvrir une expression Typst.
    #[test]
    fn le_titre_de_page_garde_ses_sauts_de_ligne_et_reste_echappe() {
        let pr = provider("lulu").unwrap();
        let mut l = livre();
        l.titre_page = "Les Heures\ncreuses".into();
        l.auteur = "Ivan #Pjig".into();
        let s = source(
            &l,
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &[],
            None,
        );
        assert!(s.contains(r"Les Heures \ creuses"), "saut de ligne perdu");
        assert!(s.contains(r"Ivan \#Pjig"), "auteur non échappé");
    }

    /// Le titre et l'auteur n'arrivent pas qu'en markup : ils entrent aussi *dans une
    /// chaîne* Typst, celle de `#set document`, et dans la ligne de commentaire qui
    /// ouvre la source. Un guillemet droit y referme la chaîne — le compilateur répond
    /// `expected comma` — et un saut de ligne fait sortir du commentaire ce qui suit,
    /// qui s'imprime alors en tête du livre. L'échappement du markup ne protège ni de
    /// l'un ni de l'autre : il laisse passer le `"` et ne touche pas aux sauts de ligne.
    #[test]
    fn un_titre_a_guillemets_ne_referme_pas_la_chaine_du_document() {
        let pr = provider("lulu").unwrap();
        let mut l = livre();
        l.titre = "Le \"quai\"\nnord".into();
        l.auteur = "Ivan \"Pjig\"".into();
        let s = source(
            &l,
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &[],
            None,
        );
        let doc = s
            .lines()
            .find(|l| l.starts_with("#set document"))
            .expect("ligne #set document");
        assert_eq!(
            doc,
            r#"#set document(title: "Le \"quai\"\nnord", author: "Ivan \"Pjig\"")"#
        );
        let entete = s.lines().next().expect("ligne de commentaire");
        assert!(
            entete.starts_with("// Intérieur") && entete.contains(r"quai\"),
            "commentaire d'en-tête coupé par le titre : {entete}"
        );
    }

    /// Une police que Typst ne connaît pas ne lève aucune erreur à la composition : il
    /// compose dans sa police par défaut, en silence. C'est ainsi que l'intérieur est
    /// resté en Libertinus Serif pendant quatre jalons. Le refus est donc ici, en
    /// amont, ou il n'est nulle part.
    #[test]
    fn une_police_hors_liste_est_refusee_et_non_substituee() {
        let i = Interieur {
            police: "Comic Sans MS".into(),
            ..Default::default()
        };
        let e = i.verifie().unwrap_err();
        assert!(
            e.contains("Comic Sans MS"),
            "l'erreur ne nomme pas la police : {e}"
        );
        assert!(
            e.contains("EB Garamond"),
            "l'erreur ne dit pas ce qui est attendu : {e}"
        );
    }

    /// Les sept polices offertes doivent toutes passer : une liste qui contient une
    /// entrée que la validation refuse est une porte fermée sur elle-même.
    #[test]
    fn les_polices_offertes_sont_toutes_acceptees() {
        for p in POLICES_TEXTE {
            let i = Interieur {
                police: (*p).into(),
                ..Default::default()
            };
            assert!(i.verifie().is_ok(), "{p} offerte mais refusée");
        }
    }

    /// La police doit être déclarée, et une seule fois. Deux `#set text(font: …)` dans
    /// la même source, c'est le second qui gagne — donc un réglage qui paraît obéi
    /// alors qu'il ne l'est pas.
    #[test]
    fn la_source_declare_la_police_du_projet_une_seule_fois() {
        let pr = provider("lulu").unwrap();
        let r = Reglage {
            gouttiere: 25.0,
            blanche: false,
        };
        let int = Interieur {
            police: "Cardo".into(),
            ..Default::default()
        };
        let s = source(&livre(), &int, pr, &r, &chapitres(), None);
        assert_eq!(s.matches("font:").count(), 1);
        assert!(s.contains(r#"font: "Cardo""#), "police du projet ignorée");
    }

    /// La rupture que l'auteur a écrite s'imprime. Elle a longtemps été perdue —
    /// deux scènes se composaient collées, en alinéas consécutifs — et le test qui
    /// figeait cette dette est celui-ci, retourné : ce qui était « à l'identique »
    /// exige désormais une différence, et la marque.
    ///
    /// La même que l'épreuve compose, pour que ce qu'on relit soit ce qui s'imprime.
    #[test]
    fn une_rupture_de_scene_s_imprime() {
        let pr = provider("lulu").unwrap();
        let r = Reglage {
            gouttiere: 25.0,
            blanche: false,
        };
        let int = Interieur::default();
        let sans = vec![Piece {
            sorte: Sorte::Chapitre(1),
            titre: "Un".into(),
            blocs: vec![
                Bloc::Paragraphe("Avant.".into()),
                Bloc::Paragraphe("Après.".into()),
            ],
        }];
        let avec = vec![Piece {
            sorte: Sorte::Chapitre(1),
            titre: "Un".into(),
            blocs: vec![
                Bloc::Paragraphe("Avant.".into()),
                Bloc::Scene,
                Bloc::Paragraphe("Après.".into()),
            ],
        }];
        let s = source(&livre(), &int, pr, &r, &avec, None);
        assert_ne!(
            source(&livre(), &int, pr, &r, &sans, None),
            s,
            "la rupture de scène est encore perdue"
        );
        assert!(s.contains(SCENE), "marque de rupture absente");
    }

    /// Le premier chapitre suit déjà le saut de page du copyright : un saut de plus
    /// laisserait une page blanche parasite, qui décalerait toute la pagination.
    #[test]
    fn le_premier_chapitre_n_ajoute_pas_de_saut_de_page() {
        let pr = provider("lulu").unwrap();
        let chs = vec![
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("A.".into())],
            },
            Piece {
                sorte: Sorte::Chapitre(2),
                titre: "Deux".into(),
                blocs: vec![Bloc::Paragraphe("B.".into())],
            },
        ];
        let s = source(
            &livre(),
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &chs,
            None,
        );
        let corps = s.split("#set page(footer: context").nth(1).unwrap();
        assert_eq!(
            corps.matches("#pagebreak()").count(),
            1,
            "un seul saut, entre les deux chapitres"
        );
    }

    /// Une dédicace renseignée coûte exactement deux pages : la belle page et sa
    /// blanche. Une seule, et le premier chapitre s'ouvrirait au verso ; trois, et le
    /// livre gagne un feuillet que personne n'a demandé — dans les deux cas le dos est
    /// faux, et il ne se découvre qu'après tirage.
    #[test]
    fn une_dedicace_ajoute_une_belle_page_et_sa_blanche() {
        let l0 = livre();
        let sans = liminaires(
            &Contexte {
                livre: &l0,
                imprimeur: None,
            },
            &Interieur::default(),
            &[],
        );
        let mut l = livre();
        l.dedicace = "À M., qui a tenu la lampe.".into();
        let avec = liminaires(
            &Contexte {
                livre: &l,
                imprimeur: None,
            },
            &Interieur::default(),
            &[],
        );

        assert_eq!(
            avec.matches("#pagebreak()").count(),
            sans.matches("#pagebreak()").count() + 2,
            "la dédicace ne coûte pas deux pages"
        );
        assert!(
            avec.contains("#align(right, emph(text(size: 9.5pt)[À M., qui a tenu la lampe.]))"),
            "la dédicace n'est pas composée en petit italique à droite : {avec}"
        );
    }

    /// Absente, vide ou faite d'espaces : la même source, à l'octet près. C'est ce qui
    /// garantit qu'un livre déjà composé ne change pas de pagination — donc pas de dos —
    /// du seul fait que le champ existe désormais.
    #[test]
    fn une_dedicace_vide_ou_blanche_ne_compose_rien() {
        let l0 = livre();
        let sans = liminaires(
            &Contexte {
                livre: &l0,
                imprimeur: None,
            },
            &Interieur::default(),
            &[],
        );
        for creux in ["", "   ", "\n \n"] {
            let mut l = livre();
            l.dedicace = creux.into();
            assert_eq!(
                liminaires(
                    &Contexte {
                        livre: &l,
                        imprimeur: None,
                    },
                    &Interieur::default(),
                    &[],
                ),
                sans,
                "« {creux:?} » a été pris pour une dédicace"
            );
        }
    }

    /// Les deux pièges déjà gardés pour le titre de page : le markup Typst doit être
    /// échappé, et les sauts de ligne voulus doivent survivre. Un `#` non échappé fait
    /// échouer la compilation du livre entier, plusieurs centaines de pages plus loin.
    #[test]
    fn une_dedicace_est_echappee_et_garde_ses_sauts_de_ligne() {
        let mut l = livre();
        l.dedicace = "À #M.,\nqui a tenu la lampe.".into();
        let s = liminaires(
            &Contexte {
                livre: &l,
                imprimeur: None,
            },
            &Interieur::default(),
            &[],
        );

        assert!(s.contains(r"À \#M.,"), "dédicace non échappée : {s}");
        assert!(
            s.contains(r"\ qui a tenu la lampe."),
            "saut de ligne perdu : {s}"
        );
    }

    /// La préface est une pièce liminaire : elle se compose avant le rétablissement du
    /// folio, donc ses pages n'en portent pas — la règle validée au cadrage.
    #[test]
    fn une_preface_se_compose_avant_le_folio() {
        let mut pieces = vec![Piece {
            sorte: Sorte::Liminaire,
            titre: "Préface".into(),
            blocs: vec![Bloc::Paragraphe("Entrez.".into())],
        }];
        pieces.extend(chapitres());
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &pieces,
            None,
        );
        let preface = s.find("Préface").expect("la préface doit être composée");
        let folio = s
            .find("#set page(footer: context")
            .expect("le folio du corps");
        assert!(
            preface < folio,
            "la préface passe après le rétablissement du folio"
        );
        assert!(s.contains("Entrez."), "le texte de la préface est perdu");
    }

    /// Une pièce liminaire ne laisse pas la suite ouvrir au verso.
    ///
    /// Sa longueur dépend d'un texte que l'auteur retouche : une préface de deux pages
    /// laisse le corps s'ouvrir en recto, la même préface d'une page l'ouvre en verso —
    /// et rien ne le dit avant tirage. Le calage pose une blanche **au verso** quand il
    /// en faut une, jamais au recto : une page vide isolée est un verso, sinon elle
    /// n'est pas une blanche, c'est une page perdue.
    #[test]
    fn une_piece_liminaire_cale_la_suite_sur_une_belle_page() {
        let piece = |t: &str| Piece {
            sorte: Sorte::Liminaire,
            titre: t.into(),
            blocs: vec![Bloc::Paragraphe("Entrez.".into())],
        };
        let l = livre();
        let s = liminaires(
            &Contexte {
                livre: &l,
                imprimeur: None,
            },
            &Interieur::default(),
            &[piece("Préface"), piece("Avant-propos")],
        );
        assert_eq!(
            s.matches(r#"#pagebreak(to: "odd", weak: true)"#).count(),
            2,
            "chaque pièce liminaire cale ce qui la suit sur un recto : {s}"
        );
    }

    /// Une partie qui ouvre le corps n'a rien à caler : les liminaires viennent de
    /// rendre la main sur une belle page vierge. Le calage y verrait une page impaire
    /// « en cours » et poserait sa blanche **au recto**, envoyant la partie au verso —
    /// le dispositif exactement à l'envers, pour deux pages payées au tirage.
    #[test]
    fn une_partie_qui_ouvre_le_corps_ne_pose_pas_de_blanche() {
        let pieces = vec![
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Avant Clément".into(),
                blocs: Vec::new(),
            },
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
        ];
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &pieces,
            None,
        );
        assert!(
            !s.contains("calc.odd(here().page())"),
            "la partie en tête de corps se cale sur une page vierge : {s}"
        );
    }

    /// Une page de partie prend une belle page au verso blanc, sans folio : deux
    /// `#page(footer: none)`. Et comme `#page` rompt le flux de lui-même, le chapitre
    /// qui suit ne doit pas ajouter un `#pagebreak()` — il laisserait une page blanche
    /// de plus, invisible à la lecture du code et payée au tirage.
    ///
    /// La comparaison porte sur un corps d'un seul chapitre : la partie **et** le
    /// chapitre qui la suit doivent, à eux deux, ne coûter aucun saut de plus.
    #[test]
    fn une_page_de_partie_prend_une_belle_page_sans_folio_et_sans_saut_en_trop() {
        let pieces = vec![
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Avant Clément".into(),
                blocs: Vec::new(),
            },
            Piece {
                sorte: Sorte::Chapitre(2),
                titre: "Deux".into(),
                blocs: vec![Bloc::Paragraphe("Suite.".into())],
            },
        ];
        let pr = provider("lulu").unwrap();
        let r = Reglage {
            gouttiere: 25.0,
            blanche: false,
        };
        let avec = source(&livre(), &Interieur::default(), pr, &r, &pieces, None);
        let sans = source(&livre(), &Interieur::default(), pr, &r, &chapitres(), None);
        assert_eq!(
            avec.matches("#page(footer: none)").count(),
            sans.matches("#page(footer: none)").count() + 2,
            "la partie doit ajouter exactement deux pages sans folio"
        );
        assert_eq!(
            avec.matches("#pagebreak()").count(),
            sans.matches("#pagebreak()").count(),
            "le chapitre qui suit la partie ne doit pas ajouter de saut"
        );
        // La casse est laissée à Typst (`#upper`), pour qu'elle suive la langue du
        // document : c'est le titre passé à `majuscules` qu'on vérifie, pas son rendu.
        assert!(
            avec.contains("#upper[Avant Clément]"),
            "titre de partie absent : {avec}"
        );
    }

    /// Une ouverture de partie est une belle page — un recto, jamais un verso. La parité,
    /// au milieu du corps, dépend de la longueur du chapitre qui précède, donc d'un texte
    /// que l'auteur retouche : sans saut de parité, trois paragraphes ajoutés au chapitre
    /// d'avant font paraître la partie au verso et sa blanche au recto, le dispositif
    /// exactement à l'envers. Le compte de pages ne le dit pas — les deux cas coûtent deux
    /// pages — et cela ne se découvrirait qu'après tirage.
    #[test]
    fn une_page_de_partie_est_forcee_sur_une_belle_page() {
        let pieces = vec![
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Avant Clément".into(),
                blocs: Vec::new(),
            },
        ];
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &pieces,
            None,
        );
        assert!(
            s.contains("if calc.odd(here().page()) { page(footer: none)[] }"),
            "la page de partie n'est pas calée sur un recto : {s}"
        );
        // La blanche du calage est posée sans folio, comme les deux pages de la partie :
        // une page vide numérotée au milieu du livre se remarque.
        assert!(
            !s.contains("#pagebreak(to: \"odd\")"),
            "le calage par saut de parité laisserait une blanche foliotée : {s}"
        );
    }

    /// Le folio appartient au corps : une postface n'en porte pas, comme la préface.
    #[test]
    fn une_annexe_se_compose_sans_folio() {
        let mut pieces = chapitres();
        pieces.push(Piece {
            sorte: Sorte::Annexe,
            titre: "Postface".into(),
            blocs: vec![Bloc::Paragraphe("Après coup.".into())],
        });
        let s = source(
            &livre(),
            &Interieur::default(),
            provider("lulu").unwrap(),
            &Reglage {
                gouttiere: 25.0,
                blanche: false,
            },
            &pieces,
            None,
        );
        let coupe = s
            .find("#set page(footer: none)")
            .expect("le folio doit être coupé");
        let postface = s.find("Postface").expect("la postface doit être composée");
        assert!(
            coupe < postface,
            "la postface se compose avant la coupure du folio"
        );
        assert!(s.contains("Après coup."));
    }

    /// La place d'un envoi ordinaire : le bas de la page de titre, là où les projets
    /// d'avant cette spec portaient le leur — le seul endroit qu'ils savaient viser.
    const PLACE: &Place = &Place {
        page: 3,
        x: 0.5,
        y: 0.80,
        taille: 0.60,
        angle: 0.0,
    };

    fn trace() -> Trace<'static> {
        Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa, qui a lu la première version.",
            },
            place: PLACE,
        }
    }

    fn image(fichier: &str) -> Trace<'_> {
        Trace {
            quoi: Quoi::Image {
                fichier: fichier.into(),
            },
            place: PLACE,
        }
    }

    /// La source d'un intérieur ordinaire, avec ou sans envoi : tout ce que ces tests
    /// comparent est ce que l'envoi y change.
    fn source_avec(envoi: Option<Trace>) -> String {
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        source(&livre(), &Interieur::default(), pr, &r, &chapitres(), envoi)
    }

    /// Le corps composé de l'envoi, en millimètres, relevé dans la source.
    ///
    /// C'est le premier `size:` du fichier : le `foreground` se pose dans le `#set
    /// page` du préambule, donc avant le `#set text` du labeur.
    fn corps_de(s: &str) -> f64 {
        let i = s.find("size: ").expect("pas de corps d'envoi") + "size: ".len();
        let j = s[i..].find("mm").expect("corps non exprimé en mm") + i;
        s[i..j].parse().expect("corps illisible")
    }

    /// Le seul `foreground` de la source, isolé du reste.
    ///
    /// Les tests de contenu doivent s'y borner : la source entière porte déjà un
    /// `justify: false` — le pavé de copyright — et un `font:` — la police de labeur —,
    /// si bien qu'un `contains` sur elle serait vrai sans le moindre envoi. Un test qui
    /// ne peut pas échouer ne protège rien.
    fn foreground_de(s: &str) -> String {
        let debut = s.find("foreground:").expect("pas de foreground");
        let fin = s[debut..].find("\n)").expect("foreground non refermé") + debut;
        s[debut..fin].to_string()
    }

    /// L'envoi se pose en `foreground` de page, conditionné au numéro de page. C'est
    /// ce qui lui interdit de créer une page — donc de déplacer la pagination, le dos
    /// et la planche — **sur n'importe quelle page**, et non plus sur la seule page de
    /// titre. Si ce test tombe, tous les packages d'envoi sont faux.
    #[test]
    fn un_envoi_se_pose_en_foreground_conditionne_a_sa_page() {
        let p = Place { page: 37, ..*PLACE };
        let s = source_avec(Some(Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa,",
            },
            place: &p,
        }));
        assert!(s.contains("foreground:"), "pas de foreground : {s}");
        assert!(
            s.contains("counter(page).get().first() == 37"),
            "la page visée n'est pas dans la condition : {s}"
        );
        // Le flux ne doit rien recevoir : un `#pagebreak` de plus, et le compte bouge.
        let sans = source_avec(None);
        assert_eq!(
            s.matches("#pagebreak()").count(),
            sans.matches("#pagebreak()").count(),
            "l'envoi a ajouté une rupture de page"
        );
    }

    /// Le `foreground` se pose au préambule, une fois : un `#set page(…)` au milieu du
    /// document ouvrirait une page. Il doit donc paraître **avant** le premier contenu.
    #[test]
    fn le_foreground_est_au_preambule() {
        let s = source_avec(Some(trace()));
        let f = s.find("foreground:").expect("pas de foreground");
        let premier_contenu = s.find("#v(42mm)").expect("pas de faux-titre");
        assert!(
            f < premier_contenu,
            "le foreground est posé après le contenu : {s}"
        );
    }

    /// Hors du `foreground`, la source ne bouge pas d'un octet : c'est ce qui garantit
    /// que tous les exemplaires d'un tirage partagent la même pagination.
    #[test]
    fn un_envoi_ne_touche_que_le_foreground() {
        let avec = source_avec(Some(trace()));
        let sans = source_avec(None);
        let debut = avec.find("foreground:").expect("pas de foreground");
        let fin = avec[debut..].find("\n)").expect("foreground non refermé") + debut;
        let ampute = format!("{}{}", &avec[..debut], &avec[fin..]);
        assert_eq!(
            ampute.replace(char::is_whitespace, ""),
            sans.replace(char::is_whitespace, ""),
            "l'envoi a modifié la source hors du foreground"
        );
    }

    /// L'échelle grossit l'objet entier, lettres comprises : tirer un coin à la souris
    /// agrandit une signature, il n'élargit pas une colonne de texte pour la laisser se
    /// recomposer. Le corps suit donc la taille.
    #[test]
    fn l_echelle_emporte_le_corps() {
        let petit = Place {
            taille: 0.30,
            ..*PLACE
        };
        let grand = Place {
            taille: 0.60,
            ..*PLACE
        };
        let sp = corps_de(&source_avec(Some(Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa,",
            },
            place: &petit,
        })));
        let sg = corps_de(&source_avec(Some(Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa,",
            },
            place: &grand,
        })));
        assert!(
            sg > sp * 1.9 && sg < sp * 2.1,
            "le corps n'a pas doublé : {sp} → {sg}"
        );
    }

    /// L'inclinaison passe par `rotate`, dont l'origine est le centre — comme en CSS,
    /// sans quoi le canevas et Typst ne montreraient pas la même chose.
    #[test]
    fn l_inclinaison_passe_par_rotate() {
        let p = Place {
            angle: -4.0,
            ..*PLACE
        };
        let s = source_avec(Some(Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa,",
            },
            place: &p,
        }));
        assert!(s.contains("rotate(-4"), "pas de rotation : {s}");
    }

    /// La main choisie doit être celle qui compose : sans le `font:`, Typst écrirait
    /// l'envoi dans la police de labeur du livre, et le mot ne ressemblerait plus à un
    /// mot écrit à la main.
    #[test]
    fn l_envoi_est_compose_dans_sa_main() {
        let s = foreground_de(&source_avec(Some(trace())));
        assert!(s.contains(r#"font: "Caveat""#), "main absente : {s}");
    }

    /// Le document est justifié — c'est bon pour trois cents pages de roman, et faux
    /// pour un mot écrit à la main : aucune main n'aligne son bord droit. Sans ce
    /// `justify: false`, l'envoi sort en pavé, ce qui trahit l'écriture manuscrite au
    /// premier coup d'œil et ne se voit dans aucun compte.
    #[test]
    fn un_envoi_n_est_pas_justifie() {
        let s = foreground_de(&source_avec(Some(trace())));
        assert!(s.contains("justify: false"), "envoi justifié : {s}");
    }

    /// Le document césure — c'est bon pour un roman justifié, et faux pour un mot écrit
    /// à la main : personne ne coupe « dif-fèrent » en tournant la ligne. Relevé sur un
    /// envoi réellement composé, pas supposé.
    #[test]
    fn un_envoi_ne_cesure_pas() {
        let s = foreground_de(&source_avec(Some(trace())));
        assert!(s.contains("hyphenate: false"), "envoi césuré : {s}");
    }

    /// Une image ne s'écrit pas dans une police : lui en imposer une reviendrait à
    /// composer du texte là où il n'y en a pas, et le mot manuscrit passerait au
    /// travers.
    #[test]
    fn une_image_d_envoi_n_emporte_aucune_police() {
        let s = foreground_de(&source_avec(Some(image("Léa.png"))));
        assert!(!s.contains("font:"), "une police s'est glissée : {s}");
        assert!(
            s.contains(r#"image("Léa.png", width: 60%)"#),
            "l'image n'est pas posée à sa taille : {s}"
        );
    }

    /// Même piège que le titre de page et que la dédicace : le markup Typst doit être
    /// échappé, les sauts de ligne voulus doivent survivre.
    #[test]
    fn un_envoi_est_echappe_et_garde_ses_sauts_de_ligne() {
        let t = Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À #Léa,\navec mon amitié.",
            },
            place: PLACE,
        };
        let s = foreground_de(&source_avec(Some(t)));

        assert!(s.contains(r"À \#Léa,"), "envoi non échappé : {s}");
        assert!(
            s.contains(r"\ avec mon amitié."),
            "saut de ligne perdu : {s}"
        );
    }

    /// **Point de sortie : le PDF de l'intérieur.** Aucun jeton ne doit survivre à la
    /// composition — un `%AUTEUR%` qui passe ici s'imprime dans le livre.
    ///
    /// Le test porte sur la source entière, et non sur le seul copyright : il doit
    /// casser le jour où un champ libre de plus est branché sans passer par son
    /// accesseur.
    #[test]
    fn aucun_jeton_ne_survit_a_la_source_de_l_interieur() {
        let mut l = livre();
        l.titre_page = "%TITRE%".into();
        l.copyright = "© %AUTEUR%, 2026.\nTous droits réservés.".into();
        l.dedicace = "Pour %AUTEUR%.".into();

        let pr = provider("bod").unwrap();
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let src = source(&l, &Interieur::default(), pr, &r, &chapitres(), None);

        for jeton in ["%TITRE%", "%AUTEUR%", "%GENRE%"] {
            assert!(!src.contains(jeton), "{jeton} a traversé la composition");
        }
        assert!(
            src.contains("Ivan Pjig"),
            "la valeur n'a pas remplacé le jeton"
        );
        assert!(src.contains("Les Heures creuses"));
    }

    /// L'objet rendu seul et l'envoi composé sur la page doivent employer **le même
    /// corps**.
    ///
    /// C'est toute la promesse du canevas : ce qu'on déplace à la souris est ce qui
    /// s'imprimera. Deux corps différents donneraient un objet dont les coupures de
    /// lignes, donc le rapport hauteur sur largeur, ne seraient pas ceux du rendu — et
    /// l'écart ne se verrait qu'après tirage.
    #[test]
    fn l_objet_du_canevas_et_l_envoi_compose_ont_le_meme_corps() {
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let place = Place {
            taille: 0.55,
            ..*PLACE
        };
        let t = Trace {
            quoi: Quoi::Texte {
                police: "Caveat",
                texte: "À Léa,",
            },
            place: &place,
        };
        let sur_la_page = corps_de(&source(
            &livre(),
            &Interieur::default(),
            pr,
            &Reglage {
                gouttiere: pr.gouttieres[0].2,
                blanche: false,
            },
            &chapitres(),
            Some(t.clone()),
        ));
        // La largeur passée à l'objet est celle qu'il occupera sur la page : c'est le
        // contrat que `envoi_objet` honore côté commandes.
        let seul = corps_de(&source_objet(&t, pr.format.0 * place.taille));
        assert_eq!(seul, sur_la_page, "le canevas ne montrera pas le rendu");
    }

    /* ---------- les repères de table ---------- */

    /// Ce qu'un repère porte, sorte par sorte. Les quatre lignes de ce test sont les
    /// quatre cas que `Sorte` admet : la table du lot 3 n'aura rien d'autre à composer.
    ///
    /// Le rang n'est pas décoratif — c'est lui qui indente. Une `Partie` rendue au
    /// second rang mettrait la partie au niveau de ses propres chapitres, et la table
    /// mentirait sur la structure du livre.
    #[test]
    fn chaque_sorte_porte_son_rang_son_numero_et_son_titre() {
        let cas = [
            (
                Sorte::Partie("II".into()),
                "Seconde",
                r#"#metadata((rang: 1, numero: "II", titre: "Seconde"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Chapitre(7),
                "Le vent",
                r#"#metadata((rang: 2, numero: "7", titre: "Le vent"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Liminaire,
                "Préface",
                r#"#metadata((rang: 2, numero: "", titre: "Préface"))<ozalid-tdm>"#,
            ),
            (
                Sorte::Annexe,
                "Postface",
                r#"#metadata((rang: 2, numero: "", titre: "Postface"))<ozalid-tdm>"#,
            ),
        ];
        for (sorte, titre, attendu) in cas {
            let p = Piece {
                sorte: sorte.clone(),
                titre: titre.into(),
                blocs: vec![],
            };
            assert_eq!(
                repere(&p).trim_end(),
                attendu,
                "le repère de {sorte:?} ne dit pas ce que la table lira"
            );
        }
    }

    /// Un chapitre sans titre est un cas admis du format (`## 7`). La table ne fabrique
    /// aucun libellé que le livre n'imprime pas : elle n'aura que le numéro à composer,
    /// et le titre vide est ce qui le lui dit.
    #[test]
    fn une_piece_sans_titre_laisse_le_titre_vide() {
        let p = Piece {
            sorte: Sorte::Chapitre(7),
            titre: String::new(),
            blocs: vec![],
        };
        assert_eq!(
            repere(&p).trim_end(),
            r#"#metadata((rang: 2, numero: "7", titre: ""))<ozalid-tdm>"#
        );
    }

    /// Un guillemet dans un titre refermerait la chaîne du dictionnaire, et la source
    /// ne composerait plus — le même piège que la page de titre, déjà tenu par
    /// `echappe_chaine`. Ici la faute serait pire : elle casserait la composition d'un
    /// livre dont le seul tort est d'avoir un titre à guillemets.
    #[test]
    fn un_titre_a_guillemets_ne_referme_pas_le_dictionnaire_du_repere() {
        let p = Piece {
            sorte: Sorte::Liminaire,
            titre: "L'« ouverture » dite\nen deux temps".into(),
            blocs: vec![],
        };
        let s = repere(&p);
        assert!(
            s.contains(r#"titre: "L'« ouverture » dite\nen deux temps""#),
            "titre mal cité : {s}"
        );
        assert_eq!(s.lines().count(), 1, "le repère tient sur une ligne : {s}");
    }

    /// Un manuscrit qui exerce les quatre ouvertures que l'intérieur compose. L'ordre
    /// est celui que `decoupe` impose : liminaires, corps, annexes.
    fn pieces_des_quatre_sortes() -> Vec<Piece> {
        vec![
            Piece {
                sorte: Sorte::Liminaire,
                titre: "Préface".into(),
                blocs: vec![Bloc::Paragraphe("Avant.".into())],
            },
            Piece {
                sorte: Sorte::Partie("I".into()),
                titre: "Première".into(),
                blocs: vec![],
            },
            Piece {
                sorte: Sorte::Chapitre(1),
                titre: "Un".into(),
                blocs: vec![Bloc::Paragraphe("Texte.".into())],
            },
            Piece {
                sorte: Sorte::Chapitre(2),
                titre: "Deux".into(),
                blocs: vec![Bloc::Paragraphe("Encore.".into())],
            },
            Piece {
                sorte: Sorte::Annexe,
                titre: "Postface".into(),
                blocs: vec![Bloc::Paragraphe("Après.".into())],
            },
        ]
    }

    fn source_des_quatre_sortes() -> String {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        source(
            &livre(),
            &Interieur::default(),
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        )
    }

    /// Chaque pièce laisse son repère, et une seule fois. Une pièce oubliée — la
    /// postface, la page de partie — manquerait dans la table sans que rien ne le dise,
    /// et c'est exactement le défaut que la spec refuse : « une préface qui a sa page
    /// d'ouverture et n'apparaît pas dans la table serait un défaut ».
    #[test]
    fn les_quatre_sortes_laissent_chacune_leur_repere_dans_l_ordre() {
        let s = source_des_quatre_sortes();
        let reperes: Vec<&str> = s.lines().filter(|l| l.contains(TDM)).collect();
        assert_eq!(
            reperes,
            vec![
                r#"#metadata((rang: 2, numero: "", titre: "Préface"))<ozalid-tdm>"#,
                r#"#metadata((rang: 1, numero: "I", titre: "Première"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "1", titre: "Un"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "2", titre: "Deux"))<ozalid-tdm>"#,
                r#"#metadata((rang: 2, numero: "", titre: "Postface"))<ozalid-tdm>"#,
            ],
            "les repères de la source ne suivent pas le manuscrit"
        );
    }

    /// **Le repère du chapitre se pose après le saut de page, jamais avant.** Écrit
    /// avant, il serait situé sur la dernière page de la pièce précédente : la table
    /// afficherait un folio d'une page trop tôt, et le lecteur ouvrirait à la fin du
    /// chapitre d'avant. Rien ne le signalerait — ni le compte de pages, ni le rendu,
    /// seulement un livre faux.
    #[test]
    fn le_repere_d_un_chapitre_suit_le_saut_de_page_qui_l_ouvre() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#pagebreak()\n#metadata((rang: 2, numero: \"2\", titre: \"Deux\"))<ozalid-tdm>"
            ),
            "le repère du chapitre 2 n'est pas collé derrière son saut de page :\n{s}"
        );
    }

    /// **Le repère de l'annexe suit la directive qui ouvre sa zone**, pas un saut de
    /// page : la première annexe n'a pas de `#pagebreak()` à elle — c'est
    /// `#set page(footer: none)` qui ouvre la zone hors folio, et le repère doit s'y
    /// coller. Un ancrage différent le situerait sur la dernière page du corps, et la
    /// table enverrait le lecteur à la fin du dernier chapitre.
    #[test]
    fn le_repere_d_une_annexe_suit_la_directive_qui_ouvre_sa_zone() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#set page(footer: none)\n#metadata((rang: 2, numero: \"\", titre: \"Postface\"))"
            ),
            "le repère de l'annexe n'ouvre pas sa page :\n{s}"
        );
    }

    /// **Le repère de la pièce liminaire ouvre sa page**, au même titre que les trois
    /// autres poses. Rien ne le garantissait jusqu'ici : le test d'ordre ne compare que
    /// les lignes portant `TDM`, filtrées de leur contexte — déplacer le repère en fin
    /// de boucle, après `blocs_typst()` ou après le saut de parité qui clôt la pièce,
    /// laisserait cette suite intacte alors que le repère se serait décalé d'une pièce.
    #[test]
    fn le_repere_d_une_piece_liminaire_ouvre_sa_page() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#pagebreak()\n\n#metadata((rang: 2, numero: \"\", titre: \"Préface\"))\
                 <ozalid-tdm>\n#v(22mm)"
            ),
            "le repère de la pièce liminaire n'ouvre pas sa page :\n{s}"
        );
    }

    /// La page de partie est composée par `#page(footer: none)[…]`, qui rompt le flux de
    /// lui-même. Le repère doit vivre **dedans** : posé avant, il serait situé sur la
    /// page précédente ; posé après, sur la blanche du verso. Dans les deux cas la table
    /// enverrait le lecteur à côté de la page de partie.
    #[test]
    fn le_repere_d_une_partie_vit_dans_sa_page() {
        let s = source_des_quatre_sortes();
        assert!(
            s.contains(
                "#page(footer: none)[\n#metadata((rang: 1, numero: \"I\", titre: \"Première\"))<ozalid-tdm>\n#v(22mm)"
            ),
            "le repère de la partie n'ouvre pas sa page :\n{s}"
        );
    }

    /* ---------- la table des matières ---------- */

    /// La source des quatre sortes sous un réglage de table donné — `source_des_quatre_sortes`
    /// avec le réglage en plus, mêmes gabarit et gouttière.
    fn source_avec_table(table: Table) -> String {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        source(
            &livre(),
            &Interieur {
                table,
                ..Interieur::default()
            },
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        )
    }

    /// **Le livre par défaut ne porte aucune table**, et pas même la requête qui
    /// l'aurait composée. C'est la garde des livres déjà composés : leur dos ne bouge
    /// pas parce que ce lot existe.
    #[test]
    fn une_table_absente_ne_compose_rien() {
        let s = source_avec_table(Table::Absente);
        assert!(
            !s.contains("Table des matières"),
            "un livre sans réglage porte une table\n{s}"
        );
        assert!(
            !s.contains(&format!("query(<{TDM}>)")),
            "un livre sans réglage interroge les repères\n{s}"
        );
    }

    /// En tête, la table vient **après le copyright et avant la préface** : le lecteur
    /// trouve le plan du livre sans traverser un texte, et la table annonce la préface
    /// elle-même. Décision de produit du 29/08 — voir le plan du lot, § décisions.
    #[test]
    fn la_table_en_tete_se_compose_apres_le_copyright_et_avant_la_preface() {
        let s = source_avec_table(Table::EnTete);
        let table = s.find("Table des matières").expect("aucune table composée");
        let copyright = s.find("©").expect("aucun pavé de copyright");
        let preface = s.find("Préface").expect("aucune préface");
        assert!(copyright < table, "la table précède le copyright\n{s}");
        assert!(table < preface, "la table suit la préface\n{s}");
    }

    /// En fin, la table ferme le volume — **après les annexes**, qui font partie du
    /// livre qu'elle indexe.
    #[test]
    fn la_table_en_fin_ferme_le_volume_apres_les_annexes() {
        let s = source_avec_table(Table::EnFin);
        let table = s.find("Table des matières").expect("aucune table composée");
        // L'annexe de `pieces_des_quatre_sortes` s'intitule « Postface » : c'est sa
        // zone qui en fait une annexe, pas son titre.
        let annexe = s.find("Postface").expect("aucune annexe");
        assert!(annexe < table, "la table précède l'annexe\n{s}");
        assert!(
            s[table..].contains(MARQUEUR),
            "la table déborde après le marqueur de fin\n{s}"
        );
    }

    /// **La table ne porte pas l'étiquette des repères**, sous peine de se lister
    /// elle-même — une ligne « Table des matières » dans la table, avec le folio de sa
    /// propre première page.
    ///
    /// Le compte des `#metadata((rang:` est la mesure juste : il vaut le nombre de
    /// pièces, table allumée ou non. Chercher l'absence de l'étiquette ne dirait rien,
    /// puisque la table doit justement l'employer dans sa requête.
    #[test]
    fn la_table_ne_se_liste_pas_elle_meme() {
        let pieces = pieces_des_quatre_sortes();
        for table in [Table::Absente, Table::EnTete, Table::EnFin] {
            let s = source_avec_table(table);
            assert_eq!(
                s.matches("#metadata((rang:").count(),
                pieces.len(),
                "{table:?} : la table s'est ajoutée aux repères\n{s}"
            );
        }
        assert!(
            source_avec_table(Table::EnTete).contains(&format!("query(<{TDM}>)")),
            "la table ne lit pas les repères"
        );
    }

    /// **La table s'ouvre en belle page**, dans les deux positions : une table qui
    /// commence au verso se lit à contre-page, et rien dans le compte de pages ne le
    /// dirait.
    ///
    /// Le saut est un `pagebreak(to: "odd", weak: true)` et non un compte à la main sur
    /// `here().page()` : c'est l'outil que les pièces liminaires emploient déjà, et pour
    /// la même raison — la table est hors folio, donc la page qu'il insère ne porte
    /// aucun numéro.
    #[test]
    fn la_table_s_ouvre_en_belle_page() {
        // Le saut de parité **collé** à l'ouverture de pièce : construite depuis
        // `ouverture_piece`, l'attente ne fige aucun littéral de mise en forme et suit
        // le gabarit de titre si celui-ci bouge.
        let attendu = format!(
            "#pagebreak(to: \"odd\", weak: true)\n{}",
            ouverture_piece(TITRE_TABLE, Interieur::default().ouverture_piece)
        );
        for table in [Table::EnTete, Table::EnFin] {
            let s = source_avec_table(table);
            assert!(
                s.contains(&attendu),
                "{table:?} : la table ne s'ouvre pas en belle page\n{s}"
            );
        }
    }

    /// La taille d'entrée règle les lignes de la table.
    ///
    /// C'est la douzième taille de `tailles()`, et la seule que
    /// `chaque_role_typographique_prend_sa_taille` ne peut pas couvrir — elle ne paraît
    /// dans aucune source tant que le réglage est absent. Le titre de la table, lui,
    /// prend `ouverture_piece`, ce que `la_table_s_ouvre_en_belle_page` vérifie déjà.
    #[test]
    fn la_taille_d_entree_regle_les_lignes_de_la_table() {
        let r = Reglage {
            gouttiere: 20.0,
            blanche: false,
        };
        let s = source(
            &livre(),
            &Interieur {
                table: Table::EnTete,
                entree_table: 22.25,
                ..Interieur::default()
            },
            provider("bod").unwrap(),
            &r,
            &pieces_des_quatre_sortes(),
            None,
        );
        assert!(
            s.contains("set text(size: 22.25pt)"),
            "les lignes de la table ignorent leur taille\n{s}"
        );
    }

    /// **La table affiche le folio du repère qu'elle lit**, sans arithmétique entre les
    /// deux.
    ///
    /// C'est le seul endroit où cette liaison se vérifie. Les tests composés de la tâche
    /// suivante lisent les repères par la même requête que la table, mais **pas ce que
    /// la table imprime** : un `- 2` glissé sur le folio les laisserait tous verts, et
    /// la table renverrait le lecteur deux pages trop tôt sur chaque entrée. Rien dans
    /// le PDF ne le dirait à qui ne compte pas les pages à la main.
    #[test]
    fn la_table_affiche_le_folio_de_chaque_repere() {
        let s = source_avec_table(Table::EnTete);
        assert!(
            s.contains(")#e.location().page()\n"),
            "la table n'affiche pas le folio de son repère, ou le retouche\n{s}"
        );
    }

    /* ---------- le témoin de l'invariant, composé pour de vrai ---------- */

    /// Un PNG minuscule mais valide : 2 × 2 pixels, deux gris.
    ///
    /// Fabriqué en dur plutôt que lu sur le disque : la variante image doit s'exercer
    /// sans dépendre d'un fichier du dépôt, et une image qu'on peut compter en octets
    /// ne cache rien.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, 0x08, 0x02, 0x00, 0x00, 0x00, 0xfd,
        0xd4, 0x9a, 0x73, 0x00, 0x00, 0x00, 0x11, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60,
        0x60, 0x60, 0x68, 0x68, 0x68, 0x60, 0x80, 0x50, 0x00, 0x10, 0x8e, 0x03, 0x01, 0x6b, 0xa0,
        0x19, 0xc2, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    /// Un manuscrit assez long pour que « page 37 » veuille dire quelque chose.
    ///
    /// `chapitres()` fait six pages : y placer un envoi ne dirait rien du cas qui compte,
    /// celui d'une page du corps, loin des liminaires où l'ancien `#place` savait déjà
    /// vivre. Quarante chapitres d'une page chacun donnent de quoi viser au milieu.
    fn manuscrit_long() -> Vec<Piece> {
        (1..=40)
            .map(|n| Piece {
                sorte: Sorte::Chapitre(n),
                titre: format!("Chapitre {n}"),
                blocs: (0..6)
                    .map(|_| {
                        Bloc::Paragraphe(
                            "Le vent tournait dans la cour, et les heures avec lui. \
                             On attendait sans savoir quoi, comme on attend toujours."
                                .into(),
                        )
                    })
                    .collect(),
            })
            .collect()
    }

    /// Le sidecar Typst et ses polices, tels que les exemples les montent.
    fn typst_de_test() -> Typst {
        Typst::new("typst").avec_polices(Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"))
    }

    /// Compose et rend le nombre de pages.
    fn pages_de(typst: &Typst, dossier: &Path, nom: &str, s: &str) -> u32 {
        std::fs::write(dossier.join(format!("{nom}.typ")), s).expect("source non écrite");
        typst
            .pages(&dossier.join(format!("{nom}.typ")))
            .expect("pagination refusée")
    }

    /// Une page rendue en PNG, telle qu'on la verrait.
    fn page_rendue(typst: &Typst, dossier: &Path, nom: &str, page: u32) -> Vec<u8> {
        let png = dossier.join(format!("{nom}-{page}.png"));
        typst
            .apercu(&dossier.join(format!("{nom}.typ")), &png, page, 40)
            .expect("rendu refusé");
        std::fs::read(&png).expect("rendu illisible")
    }

    /// **L'invariant qui tient toute la chaîne**, vérifié en composant pour de vrai.
    ///
    /// Compter les `#place` ou les `#pagebreak` dans la source ne prouve rien : c'est
    /// Typst qui décide du nombre de pages, et lui seul. Si cet invariant tombe, la
    /// pagination change, donc le dos, donc la planche — et les exemplaires partent à
    /// l'impression avec une couverture fausse, sans que rien ne le signale.
    ///
    /// Quatre pages visées, choisies pour ce qu'elles ont de différent : la première,
    /// la page de titre où l'ancien `#place` savait déjà vivre, une page du corps, et
    /// la dernière. Plus la variante image, dont la largeur en pourcentage se résout
    /// dans un `place` imbriqué dans un `foreground` — un chemin que le texte
    /// n'exerce pas.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn un_envoi_ne_cree_aucune_page_ou_qu_il_se_pose() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let livre = livre();
        let int = Interieur::default();
        let pieces = manuscrit_long();
        let sans = pages_de(
            &typst,
            dossier.path(),
            "sans",
            &source(&livre, &int, pr, &r, &pieces, None),
        );
        assert!(
            sans > 30,
            "le manuscrit de ce test est trop court pour viser une page du corps : {sans}"
        );

        std::fs::write(dossier.path().join("mot.png"), PNG).expect("image non écrite");

        for page in [1, 3, sans / 2, sans] {
            let place = Place {
                page,
                x: 0.42,
                y: 0.73,
                taille: 0.55,
                angle: -4.0,
            };
            for (nom, quoi) in [
                (
                    "texte",
                    Quoi::Texte {
                        police: "Caveat",
                        texte: "À Léa,\nces heures creuses.",
                    },
                ),
                (
                    "image",
                    Quoi::Image {
                        fichier: "mot.png".into(),
                    },
                ),
            ] {
                let s = source(
                    &livre,
                    &int,
                    pr,
                    &r,
                    &pieces,
                    Some(Trace {
                        quoi,
                        place: &place,
                    }),
                );
                let cle = format!("{nom}-{page}");
                assert_eq!(
                    pages_de(&typst, dossier.path(), &cle, &s),
                    sans,
                    "un envoi en {nom} posé page {page} a déplacé la pagination"
                );
                // Le compte de pages seul ne prouverait rien : il serait tout aussi
                // identique si l'envoi ne s'imprimait nulle part. La page visée doit
                // donc différer de la même page sans envoi — et elle seule.
                assert_ne!(
                    page_rendue(&typst, dossier.path(), &cle, page),
                    page_rendue(&typst, dossier.path(), "sans", page),
                    "un envoi en {nom} visant la page {page} ne s'y voit pas"
                );
                let ailleurs = if page == 1 { 2 } else { 1 };
                assert_eq!(
                    page_rendue(&typst, dossier.path(), &cle, ailleurs),
                    page_rendue(&typst, dossier.path(), "sans", ailleurs),
                    "un envoi visant la page {page} a débordé sur la {ailleurs}"
                );
            }
        }
    }

    /// **La preuve du lot, et son seul livrable.** Les repères ne déplacent aucune page
    /// et ne se voient sur aucune.
    ///
    /// Compter les `#metadata` dans la source ne prouverait rien : c'est Typst qui décide
    /// de la mise en page, et un élément « invisible » qui ouvrirait un paragraphe
    /// ajouterait un espacement — donc, sur un livre entier, des pages. La pagination
    /// change alors le dos, donc la planche, et les exemplaires partent avec une
    /// couverture fausse sans que rien ne le signale.
    ///
    /// La référence est la **même source privée de ses repères**, ligne à ligne : la
    /// seule différence entre les deux documents est ce que ce lot ajoute. Comparer
    /// chaque page rendue, et pas seulement le compte, ferme la porte au cas où deux
    /// écarts se compenseraient.
    ///
    /// Les deux variantes sont rendues par `Typst::apercus`, une invocation chacune :
    /// `apercu` page à page recomposerait le livre entier à chaque page, plus de
    /// quatre-vingts fois ici pour deux compositions qui suffisent.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn les_reperes_n_occupent_aucune_place_et_ne_se_voient_nulle_part() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let avec = source(
            &livre(),
            &Interieur::default(),
            pr,
            &r,
            &manuscrit_long(),
            None,
        );
        let sans: String = avec
            .lines()
            .filter(|l| !l.contains(TDM))
            .collect::<Vec<_>>()
            .join("\n");
        // `avec.lines().join("\n")` perdrait le saut de ligne final quel que soit le
        // nombre de repères : un simple `assert_ne!` resterait vert même si `repere()`
        // rendait la chaîne vide. Compter les lignes qui portent `TDM` ferme la porte.
        assert!(
            avec.lines().filter(|l| l.contains(TDM)).count() >= 40,
            "la source ne porte pas les repères attendus : rien n'est prouvé"
        );

        let n_avec = pages_de(&typst, dossier.path(), "avec", &avec);
        let n_sans = pages_de(&typst, dossier.path(), "sans", &sans);
        assert!(n_sans > 30, "manuscrit trop court pour prouver : {n_sans}");
        assert_eq!(
            n_avec, n_sans,
            "les repères ont déplacé la pagination : {n_avec} au lieu de {n_sans}"
        );

        let pages_avec = typst
            .apercus(
                &dossier.path().join("avec.typ"),
                &dossier.path().join("avec-{p}.png"),
                40,
            )
            .expect("rendu refusé");
        let pages_sans = typst
            .apercus(
                &dossier.path().join("sans.typ"),
                &dossier.path().join("sans-{p}.png"),
                40,
            )
            .expect("rendu refusé");
        assert_eq!(
            pages_avec.len(),
            pages_sans.len(),
            "les repères ont déplacé la pagination : {} pages avec, {} sans",
            pages_avec.len(),
            pages_sans.len()
        );
        for (page, (a, s)) in pages_avec.iter().zip(pages_sans.iter()).enumerate() {
            assert_eq!(
                std::fs::read(a).expect("rendu illisible"),
                std::fs::read(s).expect("rendu illisible"),
                "un repère se voit sur la page {}",
                page + 1
            );
        }
    }

    /// **Le repère est situé sur la page qu'il ouvre**, et c'est tout ce qui fera la
    /// justesse des folios de la table au lot 3. Posé un cran trop tôt, il enverrait le
    /// lecteur à la fin de la pièce précédente ; rien dans le compte de pages ni dans le
    /// rendu ne le dirait.
    ///
    /// Le manuscrit exerce **les quatre poses** — `pieces_des_quatre_sortes()` : une
    /// pièce liminaire, une page de partie, deux chapitres, une annexe, dans l'ordre que
    /// `decoupe` impose. C'est la page de partie qui porte le risque : posée à
    /// l'intérieur de son `#page(footer: none)[…]` mais mal placée dans le bloc, elle ne
    /// se verrait ni au compte de pages ni au rendu, et ce test est le seul à composer
    /// pour de vrai jusque-là. Les folios ne sont pas consécutifs — parties et annexe
    /// intercalent des pages blanches ou de parité — mais **aucun ne doit se répéter** :
    /// un repère mal ancré rend deux fois le même folio, exactement le défaut cherché.
    ///
    /// Les folios sont relevés par `Typst::mesures`, qui lit `<mesures>` sans composer de
    /// PDF : la source de test publie ce que la table interrogera, sans qu'aucune API
    /// neuve n'entre dans le code de production — la table, elle, lira ses repères depuis
    /// Typst même. Les valeurs attendues sont un relevé, pas un calcul : composées une
    /// fois, puis figées ici.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn chaque_repere_est_situe_sur_la_page_qu_il_ouvre() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces = pieces_des_quatre_sortes();
        let mut s = source(&livre(), &Interieur::default(), pr, &r, &pieces, None);
        // Le folio de chaque repère, indexé par son rang d'apparition : `mesures` rend
        // un dictionnaire de nombres, c'est exactement ce qu'il faut.
        s.push_str(
            "\n#context [#metadata(query(<ozalid-tdm>).enumerate().fold((:), (d, it) => \
             d + ((str(it.at(0))): it.at(1).location().page())))<mesures>]\n",
        );
        let chemin = dossier.path().join("ancrage.typ");
        std::fs::write(&chemin, &s).expect("source non écrite");
        let folios = typst.mesures(&chemin).expect("mesures refusées");

        let releves: Vec<f64> = (0..pieces.len())
            .map(|i| {
                *folios
                    .get(&i.to_string())
                    .unwrap_or_else(|| panic!("aucun repère au rang {i} : {folios:?}"))
            })
            .collect();
        assert_eq!(
            releves,
            vec![5.0, 7.0, 9.0, 10.0, 11.0],
            "les repères ne suivent pas l'ouverture de leur pièce"
        );
    }

    /* ---------- le témoin de l'invariant, composé pour de vrai ---------- */

    /// Les folios que la source publie sous `<mesures>`, un par repère, dans l'ordre.
    ///
    /// C'est exactement ce que la table affiche : elle lit les mêmes repères par la même
    /// requête, et rend le même `.location().page()`. Mesurer ici, plutôt que de lire la
    /// table rendue, évite de reconnaître des chiffres dans un PNG pour vérifier une
    /// valeur que Typst sait dire.
    fn folios_des_reperes(typst: &Typst, dossier: &Path, nom: &str, mut s: String) -> Vec<f64> {
        s.push_str(
            "\n#context [#metadata(query(<ozalid-tdm>).enumerate().fold((:), (d, it) => \
             d + ((str(it.at(0))): it.at(1).location().page())))<mesures>]\n",
        );
        let chemin = dossier.join(format!("{nom}.typ"));
        std::fs::write(&chemin, &s).expect("source non écrite");
        let folios = typst.mesures(&chemin).expect("mesures refusées");
        (0..folios.len())
            .map(|i| {
                *folios
                    .get(&i.to_string())
                    .unwrap_or_else(|| panic!("aucun repère au rang {i} : {folios:?}"))
            })
            .collect()
    }

    /// **La preuve du lot.** La table se compte elle-même dans les folios qu'elle
    /// affiche.
    ///
    /// C'est toute la mécanique de la spec § 2.3, et elle ne se raisonne pas : insérer
    /// une table décale les pièces qui la suivent, donc les folios qu'elle vient
    /// d'annoncer. Si Typst ne résolvait pas cette auto-référence en une invocation, la
    /// table renverrait le lecteur deux pages trop tôt — sur toutes les entrées, sans
    /// qu'aucun compte de pages ni aucun rendu ne le signale.
    ///
    /// L'écart entre les deux compositions est vérifié **constant**, et non figé à une
    /// valeur : c'est l'intention exacte — la table pousse tout le livre du même nombre
    /// de pages, celui qu'elle occupe elle-même, blanche de parité comprise. Un écart
    /// qui varierait d'une pièce à l'autre dirait que les folios ont été relevés avant
    /// l'insertion, ce que la spec écarte comme « deux passes côté Rust ».
    ///
    /// Les deux positions sont exercées : en fin, la table n'a rien à décaler et l'écart
    /// doit être **nul** sur les pièces, qui la précèdent toutes.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn la_table_se_compte_elle_meme_dans_les_folios_qu_elle_affiche() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces = pieces_des_quatre_sortes();
        let compose = |table: Table| {
            source(
                &livre(),
                &Interieur {
                    table,
                    ..Interieur::default()
                },
                pr,
                &r,
                &pieces,
                None,
            )
        };

        let sans = compose(Table::Absente);
        let n_sans = pages_de(&typst, dossier.path(), "sans", &sans);
        let f_sans = folios_des_reperes(&typst, dossier.path(), "sans-m", sans);
        assert_eq!(
            f_sans.len(),
            pieces.len(),
            "les repères du livre nu ne sont pas au complet : {f_sans:?}"
        );

        let en_tete = compose(Table::EnTete);
        let n_en_tete = pages_de(&typst, dossier.path(), "tete", &en_tete);
        let f_en_tete = folios_des_reperes(&typst, dossier.path(), "tete-m", en_tete);
        assert_eq!(
            f_en_tete.len(),
            pieces.len(),
            "la table s'est ajoutée aux repères, ou en a perdu : {f_en_tete:?}"
        );

        let ecarts: Vec<f64> = f_en_tete
            .iter()
            .zip(f_sans.iter())
            .map(|(a, s)| a - s)
            .collect();
        let decalage = ecarts[0];
        assert!(
            decalage >= 2.0,
            "la table en tête n'a poussé le livre que de {decalage} page(s) : \
             elle ne s'imprime pas, ou pas en belle page"
        );
        assert!(
            ecarts.iter().all(|e| *e == decalage),
            "la table n'a pas décalé toutes les pièces du même nombre de pages : \
             {ecarts:?} — les folios ont été relevés avant son insertion"
        );
        assert_eq!(
            f64::from(n_en_tete) - f64::from(n_sans),
            decalage,
            "le livre n'a pas grossi de ce dont la table a décalé les pièces : \
             {n_en_tete} pages contre {n_sans}"
        );

        let en_fin = compose(Table::EnFin);
        let n_en_fin = pages_de(&typst, dossier.path(), "fin", &en_fin);
        let f_en_fin = folios_des_reperes(&typst, dossier.path(), "fin-m", en_fin);
        assert_eq!(
            f_en_fin, f_sans,
            "une table en fin a déplacé des pièces qui la précèdent toutes"
        );
        assert!(
            n_en_fin > n_sans,
            "une table en fin n'a ajouté aucune page : {n_en_fin} contre {n_sans}"
        );
        // **La belle page se prouve ici, et nulle part ailleurs en composant.** En tête,
        // le copyright rend toujours la main sur une impaire : un saut simple donnerait
        // le même livre, et le saut de parité y est une garde sans effet observable. En
        // fin, la parité dépend de la longueur des annexes — le livre nu s'arrête sur
        // une impaire, la table doit donc sauter la paire qui suit.
        assert_eq!(
            n_sans % 2,
            1,
            "ce test ne prouve la belle page que sur un livre dont la pagination nue est \
             impaire ; elle vaut {n_sans}. Allonger le manuscrit de test d'une page — ne \
             pas retirer cette garde, elle est ce qui empêche le test de devenir muet."
        );
        let ouverture = n_sans + 2;
        assert!(
            n_en_fin >= ouverture,
            "la table en fin s'est ouverte au verso : le livre fait {n_en_fin} pages, \
             elle devait s'ouvrir en {ouverture}"
        );
    }

    /// Une table longue ne dérange **ni la pagination ni l'ancrage des repères**.
    ///
    /// Le cas court d'à côté ne l'exerce pas : cinq entrées tiennent sur une page, et
    /// une table qui insérerait un saut parasite tous les N blocs y passerait inaperçue.
    /// Quarante chapitres d'une page font déborder la table, et les folios doivent
    /// rester **consécutifs** à partir de sa sortie.
    ///
    /// **Ce que ce test ne prouve pas, et qu'aucun test Rust ne peut prouver :** que les
    /// folios *imprimés dans la table* tiennent compte des pages qu'elle occupe.
    /// `typst query` refuse `text` comme `block` — « is not locatable » —, si bien que
    /// le seul élément interrogeable est le repère, dont le folio est juste que la table
    /// converge ou non. Le point fixe de la spec § 2.3 a été vérifié par composition le
    /// 29/08, sur PNG, et reste une vérification à l'œil ; la liaison entre le repère et
    /// ce qui s'imprime est gardée, elle, par
    /// `la_table_affiche_le_folio_de_chaque_repere`.
    #[test]
    #[ignore = "lance le sidecar Typst : cargo test -- --ignored"]
    fn une_table_longue_ne_derange_pas_l_ancrage_des_reperes() {
        let typst = typst_de_test();
        let dossier = tempfile::tempdir().expect("répertoire de travail");
        let pr = provider("kdp-5x8").expect("gabarit kdp-5x8");
        let r = Reglage {
            gouttiere: pr.gouttieres[0].2,
            blanche: false,
        };
        let pieces = manuscrit_long();
        let s = source(
            &livre(),
            &Interieur {
                table: Table::EnTete,
                ..Interieur::default()
            },
            pr,
            &r,
            &pieces,
            None,
        );
        let folios = folios_des_reperes(&typst, dossier.path(), "longue", s);
        assert_eq!(folios.len(), pieces.len());
        let premier = folios[0];
        assert!(
            premier >= 7.0,
            "la table n'a pas repoussé le premier chapitre : il ouvre en {premier}"
        );
        let attendus: Vec<f64> = (0..pieces.len()).map(|i| premier + i as f64).collect();
        assert_eq!(
            folios, attendus,
            "une table longue a dérangé l'ancrage : les folios ne sont plus consécutifs"
        );
    }
}
