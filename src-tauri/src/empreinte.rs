//! Ce qui dit qu'un package n'est plus celui du projet qu'on a sous les yeux.
//!
//! Deux empreintes par livrable — l'intérieur, la couverture — parce qu'elles répondent à
//! deux questions différentes : laquelle des deux moitiés a bougé, et un intérieur déjà
//! composé peut-il resservir alors que la couverture, elle, a changé.

/// Un condensé FNV-1a 64 bits, en seize caractères hexadécimaux.
///
/// **Écrit ici et non repris de `commands::empreinte`**, qui repose sur `DefaultHasher`.
/// Celle-là nomme un répertoire de rendus : une valeur qui change fabrique un répertoire
/// neuf et l'on recalcule, personne ne le voit. Celle-ci est écrite dans le `.ozalid` et
/// relue par un binaire recompilé — or la bibliothèque standard ne garantit pas que
/// `DefaultHasher` rende la même valeur d'une version de Rust à l'autre. Une mise à jour de
/// l'application marquerait alors tous les packages périmés d'un coup, sans que rien ne
/// l'explique. FNV-1a, lui, est une spécification : il ne bougera jamais.
pub fn condense(octets: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for o in octets {
        h ^= u64::from(*o);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

use crate::projet::{Livrable, Projet};

/// L'empreinte de ce qui compose l'**intérieur** de ce livrable.
///
/// Le gabarit y entre par `Resolu::empreinte` — format, marges, gouttières —, la même
/// valeur que `Mesure::empreinte` retient déjà depuis le lot 2 du catalogue. Un gabarit que
/// le catalogue ne porte plus rend une empreinte vide : le livrable paraîtra périmé, ce qui
/// est vrai, et `normalise` l'élaguera à la prochaine ouverture.
pub fn interieur(projet: &Projet, l: &Livrable) -> String {
    let gabarit = crate::catalogue::resout(&l.fabrication)
        .map(|r| r.empreinte())
        .unwrap_or_default();
    condense(
        [
            condense(projet.texte.as_bytes()),
            json(&projet.meta.livre),
            json(&projet.meta.manuscrit),
            json(&projet.meta.interieur),
            gabarit,
        ]
        .join("|")
        .as_bytes(),
    )
}

/// L'empreinte de ce qui compose la **planche** de ce livrable.
///
/// Le livre y figure comme dans l'autre : la couverture cite `%TITRE%` et `%AUTEUR%`, et
/// l'oublier ici laisserait une moitié du livre à jour et l'autre fausse.
///
/// Le papier et la pagination y figurent parce que le **dos** en découle. On y met les deux
/// plutôt que le dos calculé : le dos est une fonction pure de ces deux-là, et le calculer
/// ici obligerait ce module à connaître `planche`, qu'il n'a aucune raison de connaître.
pub fn couverture(projet: &Projet, l: &Livrable) -> String {
    let images: Vec<String> = projet
        .images
        .iter()
        .map(|(nom, octets)| format!("{nom}:{}", condense(octets)))
        .collect();
    let pages = projet
        .meta
        .livraison
        .mesure(&l.fabrication.cle_gabarit())
        .map(|m| m.pages.to_string())
        .unwrap_or_default();
    condense(
        [
            json(&projet.meta.livre),
            json(&projet.meta.couverture),
            images.join(","),
            l.fabrication.papier.clone(),
            l.dos_mm.map(|d| d.to_string()).unwrap_or_default(),
            pages,
        ]
        .join("|")
        .as_bytes(),
    )
}

/// La forme sérialisée d'un morceau de métadonnées, pour le condenser.
///
/// `serde_json` et non `toml` : TOML exige que les valeurs précèdent les tables, et refuse
/// donc certaines structures qu'on veut seulement décrire. Le JSON n'a pas cette contrainte,
/// et il est déjà en dépendance.
///
/// Une erreur devient un morceau au lieu d'une panique : cette fonction est appelée à chaque
/// vue, et faire tomber l'application pour un condensé serait hors de proportion. Rendre une
/// chaîne vide serait pire — le morceau cesserait silencieusement de compter.
fn json<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| format!("!{e}"))
}

/// Où en est un livrable, comparé à l'état courant du projet.
///
/// `Serialize` parce que le lot 2 le fera descendre dans la vue que le front consomme ; il
/// n'a aucun autre usage côté Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Etat {
    /// Jamais généré : rien à regarder, rien à refaire tant qu'on ne l'a pas demandé.
    Jamais,
    /// La dernière génération a échoué ; son message est dans `Generation::Echec`.
    Echec,
    AJour,
    Perime {
        interieur: bool,
        couverture: bool,
    },
}

/// Où en est ce livrable.
///
/// Deux empreintes recalculées à chaque appel, sans cache. Hacher le manuscrit du témoin
/// coûte quelques dixièmes de milliseconde là où composer coûte des secondes ; un cache
/// achèterait ce dixième-là au prix d'une invalidation à tenir juste — le même arbitrage que
/// `commands::envoi_vignettes` a déjà tranché dans ce sens.
pub fn etat(projet: &Projet, l: &Livrable) -> Etat {
    let (i, c) = match &l.generation {
        crate::projet::Generation::Jamais => return Etat::Jamais,
        crate::projet::Generation::Echec { .. } => return Etat::Echec,
        crate::projet::Generation::Fait {
            interieur: i,
            couverture: c,
        } => (i, c),
    };
    let (di, dc) = (*i != interieur(projet, l), *c != couverture(projet, l));
    if di || dc {
        Etat::Perime {
            interieur: di,
            couverture: dc,
        }
    } else {
        Etat::AJour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les valeurs sont **gelées**, et c'est tout l'intérêt du test : cette empreinte est
    /// écrite dans le `.ozalid` et relue par un binaire qu'on aura recompilé entre-temps.
    /// Un algorithme qu'on changerait sans y penser marquerait d'un coup tous les packages
    /// de tous les projets comme périmés, sans que rien à l'écran puisse l'expliquer. Les
    /// trois vecteurs sont ceux de la spécification FNV-1a 64 bits.
    #[test]
    fn le_condense_est_gele() {
        assert_eq!(condense(b""), "cbf29ce484222325");
        assert_eq!(condense(b"a"), "af63dc4c8601ec8c");
        assert_eq!(condense(b"ozalid"), "dc0fb47ed8d84474");
    }

    /// Deux entrées voisines ne se confondent pas : sans quoi changer une lettre du titre
    /// laisserait la couverture marquée à jour.
    #[test]
    fn deux_entrees_voisines_ne_se_condensent_pas_pareil() {
        assert_ne!(condense(b"a"), condense(b"b"));
    }

    /// Ce qui compose l'intérieur le périme, et rien d'autre. Le manuscrit fait la
    /// pagination ; l'identité du livre fait la page de titre et les liminaires ; les
    /// réglages font la police et les corps ; le gabarit fait la boîte.
    #[test]
    fn l_empreinte_d_interieur_suit_ce_qui_compose_l_interieur() {
        let p = projet_d_essai();
        let l = p.meta.livraison.livrables[0].clone();
        let depart = interieur(&p, &l);

        let mut q = p.clone();
        q.texte.push_str("\n\nUn paragraphe de plus.");
        assert_ne!(interieur(&q, &l), depart, "le manuscrit doit périmer");

        let mut q = p.clone();
        q.meta.livre.titre = "Un autre titre".into();
        assert_ne!(interieur(&q, &l), depart, "le titre doit périmer");

        let mut q = p.clone();
        q.meta.interieur.corps = 11.0;
        assert_ne!(interieur(&q, &l), depart, "le corps doit périmer");
    }

    /// Et ce qui ne le compose pas ne le périme pas : la couverture retouchée ou un envoi
    /// ajouté ne changent pas un octet de l'intérieur. Sans ce bord, la liste crierait au
    /// loup et on cesserait de la lire.
    #[test]
    fn l_empreinte_d_interieur_ignore_la_couverture_et_les_envois() {
        let p = projet_d_essai();
        let l = p.meta.livraison.livrables[0].clone();
        let depart = interieur(&p, &l);

        let mut q = p.clone();
        q.meta.couverture.maquette = None;
        assert_eq!(
            interieur(&q, &l),
            depart,
            "la couverture ne périme pas l'intérieur"
        );

        let mut q = p.clone();
        q.meta.envois.liste.push(crate::envoi::Envoi::default());
        assert_eq!(
            interieur(&q, &l),
            depart,
            "un envoi ne périme pas l'intérieur"
        );
    }

    /// La couverture porte le dos, donc le papier et la pagination : un changement de
    /// police repagine, le dos bouge, et la planche déjà écrite est fausse. Sans ces deux
    /// morceaux, elle se dirait à jour — c'est le risque nommé au § 8 de la spec.
    #[test]
    fn l_empreinte_de_couverture_suit_le_dos() {
        let p = projet_d_essai();
        let l = p.meta.livraison.livrables[0].clone();
        let depart = couverture(&p, &l);

        let mut autre_papier = l.clone();
        autre_papier.fabrication.papier = "blanc-90".into();
        assert_ne!(
            couverture(&p, &autre_papier),
            depart,
            "le papier fait le dos"
        );

        let mut q = p.clone();
        q.meta.livraison.retenir_mesure(
            &l.fabrication.cle_gabarit(),
            crate::projet::Mesure {
                pages: 400,
                ..mesure_d_essai()
            },
        );
        assert_ne!(couverture(&q, &l), depart, "la pagination fait le dos");

        let mut q = p.clone();
        q.meta.couverture.maquette = None;
        assert_ne!(couverture(&q, &l), depart, "la maquette fait la planche");

        let mut q = p.clone();
        q.images.insert("premiere.jpg".into(), vec![1, 2, 3]);
        assert_ne!(couverture(&q, &l), depart, "l'image fait la planche");
    }

    /// Le manuscrit ne touche pas la planche : c'est ce qui permet, au lot 2, de recomposer
    /// une couverture sans recomposer l'intérieur — et l'inverse.
    #[test]
    fn l_empreinte_de_couverture_ignore_le_manuscrit() {
        let p = projet_d_essai();
        let l = p.meta.livraison.livrables[0].clone();
        let depart = couverture(&p, &l);
        let mut q = p.clone();
        q.texte.push_str("\n\nUn paragraphe de plus.");
        assert_eq!(
            couverture(&q, &l),
            depart,
            "le manuscrit ne périme la couverture que par la pagination, retenue à part"
        );
    }

    /// Les quatre réponses, sur le même projet. C'est cette fonction que l'écran du lot 3
    /// interrogera pour marquer une ligne, et le message qu'il affichera dépend de
    /// *laquelle* des deux moitiés a bougé — d'où le couple de booléens plutôt qu'un simple
    /// « périmé ».
    #[test]
    fn l_etat_dit_laquelle_des_deux_moities_a_bouge() {
        let mut p = projet_d_essai();
        let l = p.meta.livraison.livrables[0].clone();

        assert_eq!(etat(&p, &l), Etat::Jamais, "rien n'a été généré");

        let mut a_jour = l.clone();
        a_jour.generation = crate::projet::Generation::Fait {
            interieur: interieur(&p, &l),
            couverture: couverture(&p, &l),
        };
        assert_eq!(etat(&p, &a_jour), Etat::AJour);

        // La couverture seule bouge : l'intérieur écrit reste bon, et le lot 2 s'en servira
        // pour ne pas recomposer 258 pages afin de changer une image.
        let mut q = p.clone();
        q.images.insert("premiere.jpg".into(), vec![1, 2, 3]);
        assert_eq!(
            etat(&q, &a_jour),
            Etat::Perime {
                interieur: false,
                couverture: true
            }
        );

        // Le manuscrit bouge : l'intérieur est faux, et la couverture le deviendra dès que
        // la pagination aura été reprise — mais tant qu'elle ne l'a pas été, la mesure
        // retenue n'a pas changé, et seule la moitié intérieure est en cause.
        p.texte.push_str("\n\nUn paragraphe de plus.");
        assert_eq!(
            etat(&p, &a_jour),
            Etat::Perime {
                interieur: true,
                couverture: false
            }
        );
    }

    /// Un échec retenu ne se compare pas : il n'y a pas d'empreinte à confronter, et la
    /// ligne doit dire « ça n'a pas marché », pas « c'est périmé ».
    #[test]
    fn un_echec_retenu_ne_se_compare_pas() {
        let p = projet_d_essai();
        let mut l = p.meta.livraison.livrables[0].clone();
        l.generation = crate::projet::Generation::Echec {
            message: "typst absent".into(),
        };
        assert_eq!(etat(&p, &l), Etat::Echec);
    }

    /// `Livre` ne dérive pas `Default` : ses quatorze champs se posent un à un, comme
    /// `package::tests::livre_d_essai` et `projet::tests::livre` le font déjà.
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

    fn projet_d_essai() -> crate::projet::Projet {
        let mut p =
            crate::projet::Projet::nouveau(livre_d_essai(), "## 01 - Un\n\nParagraphe.".into());
        p.meta.couverture.maquette = Some(
            crate::maquettes::par_cle(None, "filets")
                .expect("maquette fournie « filets »")
                .couverture,
        );
        let cle = p.meta.livraison.livrables[0].fabrication.cle_gabarit();
        p.meta.livraison.retenir_mesure(&cle, mesure_d_essai());
        p
    }

    fn mesure_d_essai() -> crate::projet::Mesure {
        crate::projet::Mesure {
            pages: 98,
            gouttiere: 14.0,
            blanche: false,
            empreinte: None,
            polices_introuvables: Vec::new(),
        }
    }
}
