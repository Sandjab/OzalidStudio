//! Les jetons `%CLE%` des champs libres du livre.
//!
//! Un champ libre — le titre de la page de titre, la dédicace, le copyright — peut
//! citer un champ clé du livre, ou l'un des axes du livrable visé. La substitution se
//! fait **à la composition**, jamais à la saisie : le `.ozalid` conserve le texte à
//! jetons, qui doit suivre le livre si le titre change.

use crate::projet::Livre;

/// Les axes du livrable visé, tels que le catalogue les **nomme**.
///
/// À ne pas confondre avec [`crate::catalogue::Fabrication`], qui porte les mêmes axes
/// en **clés** — `creme-90` là où celui-ci dit « Crème 90 g ». Les clés identifient et
/// se comparent ; ces noms-là s'impriment, et c'est tout leur emploi : ils marquent
/// l'exemplaire qu'on tient en main de l'imprimeur, du format et du papier qui l'ont
/// fait. La reliure n'y est pas — le quatrième axe n'a encore trouvé aucun usage, et
/// un jeton qu'on n'écrit nulle part est une table à tenir sans contrepartie.
///
/// Chaque axe est `Option` pour la même raison : il y a des compositions où il n'a pas
/// de sens. L'ebook n'est imprimé nulle part et n'a pas de papier ; il a un format,
/// puisqu'il vient d'un gabarit. Un axe absent rend la chaîne **vide**, jamais le jeton
/// littéral — un `%PAPIER%` composé en toutes lettres serait la faute que ces jetons
/// existent pour éviter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Marques<'a> {
    pub imprimeur: Option<&'a str>,
    pub format: Option<&'a str>,
    pub papier: Option<&'a str>,
}

impl<'a> Marques<'a> {
    /// Les marques d'un tirage chez un imprimeur, sans son papier.
    ///
    /// Le raccourci des appelants qui n'ont qu'un [`crate::catalogue::Provider`] sous la
    /// main : il porte le nom du POD et celui du format, jamais celui du papier — un
    /// provider vaut pour un triplet, et plusieurs papiers s'y rattachent. Qui connaît
    /// le papier du livrable ajoute [`Marques::sur`] derrière.
    pub fn chez(pr: &'a crate::catalogue::Provider) -> Self {
        Self {
            imprimeur: Some(&pr.pod_nom),
            format: Some(&pr.format_nom),
            papier: None,
        }
    }

    /// Les mêmes marques, le papier nommé.
    pub fn sur(self, papier: &'a str) -> Self {
        Self {
            papier: Some(papier),
            ..self
        }
    }
}

/// Ce contre quoi un champ libre se résout : le livre, et les axes du livrable quand la
/// composition en vise un.
///
/// **Une seule porte.** Garder `substituer(&str, &Livre)` et ajouter une seconde fonction
/// pour les axes du livrable aurait ouvert deux chemins de substitution, dont le second
/// serait tôt ou tard oublié sur un champ libre — et l'oubli ne se verrait qu'imprimé.
pub struct Contexte<'a> {
    pub livre: &'a Livre,
    pub marques: Marques<'a>,
}

/// Un jeton et ce qu'il désigne dans le contexte.
type Jeton = (&'static str, for<'a> fn(&'a Contexte<'a>) -> &'a str);

/// Les jetons reconnus, et ce que chacun désigne.
///
/// Les huit premiers sont des clés du livre, littérales par définition : aucune n'est
/// elle-même substituée, et c'est ce qui rend toute référence cyclique impossible. Les
/// trois derniers ne changent rien à cette propriété — l'ISBN et le dépôt légal sont des
/// clés comme les autres, et les axes du livrable ne viennent pas du livre du tout.
///
/// **`%FORMAT%` et `%PAPIER%` entrent aussi dans l'empreinte de l'intérieur**, par la clé
/// de fabrication que `empreinte::interieur` y met : sans elle, deux papiers du même
/// gabarit — qui partagent une mesure par construction — partageraient une empreinte, et
/// le second se dirait à jour sur le PDF du premier.
const JETONS: [Jeton; 11] = [
    ("%TITRE%", |c| &c.livre.titre),
    ("%AUTEUR%", |c| &c.livre.auteur),
    ("%GENRE%", |c| &c.livre.genre),
    ("%EDITEUR%", |c| &c.livre.editeur),
    ("%COLLECTION%", |c| &c.livre.collection),
    ("%MONOGRAMME%", |c| &c.livre.monogramme),
    ("%ISBN%", |c| &c.livre.isbn),
    ("%DEPOT_LEGAL%", |c| &c.livre.depot_legal),
    ("%IMPRIMEUR%", |c| c.marques.imprimeur.unwrap_or("")),
    ("%FORMAT%", |c| c.marques.format.unwrap_or("")),
    ("%PAPIER%", |c| c.marques.papier.unwrap_or("")),
];

/// Les jetons reconnus, dans l'ordre où l'aide les présente.
///
/// Servie au front plutôt que recopiée dans le HTML : la table a grossi deux fois en
/// deux lots, et une copie aurait menti les deux fois.
pub fn jetons() -> Vec<&'static str> {
    JETONS.iter().map(|(j, _)| *j).collect()
}

/// Remplace les jetons connus par la valeur de leur champ clé.
///
/// **Une seule passe.** Le texte est parcouru une fois de gauche à droite : ce qu'un
/// jeton produit est poussé dans la sortie et jamais réexaminé.
///
/// Ce n'est pas une garde contre les références cycliques — il ne peut pas y en avoir,
/// un jeton ne désignant qu'une clé et une clé n'étant jamais substituée. C'est une
/// garde contre la relecture de la sortie : un `replace` par jeton en boucle aurait
/// l'air équivalent et ne l'est pas, car il traiterait la valeur du jeton précédent
/// comme du texte à substituer. Un titre valant « 100 % coton » suffit à le montrer.
///
/// Un jeton inconnu est recopié tel quel.
pub fn substituer(texte: &str, ctx: &Contexte) -> String {
    let mut sortie = String::with_capacity(texte.len());
    let mut reste = texte;
    while let Some(i) = reste.find('%') {
        sortie.push_str(&reste[..i]);
        let a_partir_du_pour_cent = &reste[i..];
        match JETONS
            .iter()
            .find(|(jeton, _)| a_partir_du_pour_cent.starts_with(jeton))
        {
            Some((jeton, valeur)) => {
                sortie.push_str(valeur(ctx));
                reste = &a_partir_du_pour_cent[jeton.len()..];
            }
            None => {
                sortie.push('%');
                reste = &a_partir_du_pour_cent[1..];
            }
        }
    }
    sortie.push_str(reste);
    sortie
}

#[cfg(test)]
mod tests {
    use super::*;

    fn livre() -> Livre {
        Livre {
            titre: "Les Heures creuses".into(),
            auteur: "Ivan Pjig".into(),
            genre: "roman".into(),
            ..Livre::vide()
        }
    }

    /// Les trois axes du livrable se citent, et chacun rend **le sien**.
    ///
    /// Un par un et non tous ensemble dans une chaîne : trois champs voisins du même type
    /// se recopient de travers sans qu'aucune assertion globale ne le voie — c'est
    /// exactement le défaut qu'un `%PAPIER%` rendant le format produirait, et il ne se
    /// verrait qu'imprimé.
    #[test]
    fn chaque_axe_du_livrable_rend_le_sien() {
        let l = livre();
        let m = Marques {
            imprimeur: Some("BoD"),
            format: Some("13,5 × 21,5 cm"),
            papier: Some("Crème 90 g"),
        };
        assert_eq!(substituer("%IMPRIMEUR%", &ctx_tirage(&l, m)), "BoD");
        assert_eq!(substituer("%FORMAT%", &ctx_tirage(&l, m)), "13,5 × 21,5 cm");
        assert_eq!(substituer("%PAPIER%", &ctx_tirage(&l, m)), "Crème 90 g");
    }

    /// Un axe que la composition ne vise pas s'efface, et ne reste pas littéral.
    ///
    /// C'est la règle déjà tenue par `%IMPRIMEUR%`, étendue aux deux autres : un
    /// `%PAPIER%` composé en toutes lettres sur une 4ème serait la faute que ces jetons
    /// existent pour éviter. Le cas est réel — l'EPUB n'a ni imprimeur ni papier.
    #[test]
    fn un_axe_absent_s_efface_au_lieu_de_rester_litteral() {
        let l = livre();
        let m = Marques {
            format: Some("13,5 × 21,5 cm"),
            ..Marques::default()
        };
        assert_eq!(
            substituer("[%IMPRIMEUR%][%PAPIER%][%FORMAT%]", &ctx_tirage(&l, m)),
            "[][][13,5 × 21,5 cm]"
        );
    }

    #[test]
    fn chaque_jeton_prend_la_valeur_de_sa_cle() {
        let l = livre();
        assert_eq!(substituer("%TITRE%", &ctx(&l, None)), "Les Heures creuses");
        assert_eq!(substituer("%AUTEUR%", &ctx(&l, None)), "Ivan Pjig");
        assert_eq!(substituer("%GENRE%", &ctx(&l, None)), "roman");
    }

    #[test]
    fn un_texte_sans_jeton_ne_bouge_pas() {
        assert_eq!(
            substituer("Tous droits réservés.", &ctx(&livre(), None)),
            "Tous droits réservés."
        );
    }

    #[test]
    fn plusieurs_jetons_dans_une_phrase() {
        assert_eq!(
            substituer("%TITRE%, un %GENRE% de %AUTEUR%.", &ctx(&livre(), None)),
            "Les Heures creuses, un roman de Ivan Pjig.",
        );
    }

    /// Un jeton inconnu traverse intact : il se voit dans l'aperçu et sur l'épreuve.
    /// Le supprimer ferait disparaître du texte sans laisser de trace.
    #[test]
    fn un_jeton_inconnu_reste_tel_quel() {
        assert_eq!(
            substituer("%TITER% et 100 %", &ctx(&livre(), None)),
            "%TITER% et 100 %"
        );
    }

    /// **Le test qui compte.** Aucun cycle n'est possible — un jeton ne désigne qu'une
    /// clé, et une clé n'est jamais substituée. Le risque est ailleurs : une valeur de
    /// clé peut *contenir* ce qui ressemble à un jeton, sans rien désigner du tout.
    /// « 100 % coton » est un titre légitime. Relire la sortie ferait dire au copyright
    /// autre chose que ce qui est écrit dans le champ.
    #[test]
    fn une_valeur_qui_ressemble_a_un_jeton_reste_litterale() {
        let l = Livre {
            titre: "%AUTEUR%".into(),
            auteur: "Ivan Pjig".into(),
            ..Livre::vide()
        };
        assert_eq!(substituer("%TITRE%", &ctx(&l, None)), "%AUTEUR%");
    }

    /// Un pour-cent isolé, une paire vide, un jeton tronqué : rien ne doit paniquer
    /// ni manger le texte qui suit.
    #[test]
    fn les_pour_cent_isoles_survivent() {
        let l = livre();
        assert_eq!(substituer("100 % coton", &ctx(&l, None)), "100 % coton");
        assert_eq!(substituer("%%", &ctx(&l, None)), "%%");
        assert_eq!(substituer("%TITRE", &ctx(&l, None)), "%TITRE");
    }

    /// Les trois clés qui montent au lot 2 sont des jetons comme les autres.
    #[test]
    fn les_cles_de_la_maison_sont_des_jetons() {
        let l = Livre {
            editeur: "Ozalid".into(),
            collection: "Les Heures".into(),
            monogramme: "O".into(),
            ..livre()
        };
        assert_eq!(
            substituer("%EDITEUR%, %COLLECTION%, %MONOGRAMME%", &ctx(&l, None)),
            "Ozalid, Les Heures, O"
        );
    }

    /// La liste des jetons est servie par le Rust, seul à la connaître. La recopier
    /// dans le HTML la ferait mentir le jour où une clé s'ajoute — ce qui vient
    /// d'arriver deux fois.
    #[test]
    fn les_jetons_annonces_sont_ceux_qui_substituent() {
        let l = Livre {
            titre: "T".into(),
            auteur: "A".into(),
            genre: "G".into(),
            editeur: "E".into(),
            collection: "C".into(),
            monogramme: "M".into(),
            ..Livre::vide()
        };
        for jeton in jetons() {
            assert_ne!(
                substituer(jeton, &ctx(&l, None)),
                jeton,
                "{jeton} est annoncé mais ne substitue rien"
            );
        }
        assert_eq!(jetons().len(), JETONS.len());
    }

    fn ctx<'a>(l: &'a Livre, imprimeur: Option<&'a str>) -> Contexte<'a> {
        Contexte {
            livre: l,
            marques: Marques {
                imprimeur,
                ..Marques::default()
            },
        }
    }

    /// Le contexte d'un tirage nommé de bout en bout : les trois axes renseignés.
    fn ctx_tirage<'a>(l: &'a Livre, marques: Marques<'a>) -> Contexte<'a> {
        Contexte { livre: l, marques }
    }

    #[test]
    fn l_isbn_et_le_depot_legal_se_citent() {
        let l = Livre {
            isbn: "978-2-07-041311-9".into(),
            depot_legal: "septembre 2026".into(),
            ..Livre::vide()
        };
        assert_eq!(
            substituer("ISBN %ISBN% — dépôt légal : %DEPOT_LEGAL%", &ctx(&l, None)),
            "ISBN 978-2-07-041311-9 — dépôt légal : septembre 2026"
        );
    }

    /// L'imprimeur ne vient pas du livre : il vient de ce qu'on fabrique. Le même livre
    /// composé chez deux imprimeurs porte deux mentions.
    #[test]
    fn l_imprimeur_vient_du_contexte_pas_du_livre() {
        let l = livre();
        assert_eq!(
            substituer("Imprimé par %IMPRIMEUR%", &ctx(&l, Some("BoD"))),
            "Imprimé par BoD"
        );
    }

    /// Sans imprimeur — l'ebook, la couverture —, le jeton rend la **chaîne vide**, jamais
    /// lui-même. Un `%IMPRIMEUR%` en toutes lettres dans le pavé de copyright serait une
    /// faute que le lecteur verrait, sur un fichier que plus personne ne relit.
    #[test]
    fn sans_imprimeur_le_jeton_s_efface_au_lieu_de_rester() {
        let l = livre();
        assert_eq!(
            substituer("Imprimé par %IMPRIMEUR%.", &ctx(&l, None)),
            "Imprimé par ."
        );
    }

    /// Un ISBN vide n'écrit rien, pas le jeton : c'est le cas d'un tirage privé, et il est
    /// légitime.
    #[test]
    fn un_isbn_vide_n_ecrit_rien() {
        let l = Livre {
            isbn: String::new(),
            ..Livre::vide()
        };
        assert_eq!(substituer("%ISBN%", &ctx(&l, None)), "");
    }
}
