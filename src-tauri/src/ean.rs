//! L'ISBN d'un livre, et le symbole EAN-13 qui le porte sur la 4ème.
//!
//! Un module sans dépendance : la norme tient en trois tables et une clé de contrôle, et
//! une caisse de librairie ne lit pas un ISBN, elle lit des barres.
//!
//! Le partage est celui du reste du dépôt — ce qui se vérifie ici ne se vérifie pas
//! ailleurs. [`Isbn::lire`] refuse tout ce qui n'est pas un ISBN ; [`modules`] ne connaît
//! que treize chiffres et n'a aucune raison de les juger.

/// Les sept modules de chaque chiffre, jeu **L** — celui de la moitié gauche en parité
/// impaire. Les deux autres jeux s'en déduisent : `R` est son complément, `G` le miroir
/// de `R`. Les écrire tous les trois inviterait la faute de frappe que rien ne verrait.
const L: [[bool; 7]; 10] = [
    [false, false, false, true, true, false, true],
    [false, false, true, true, false, false, true],
    [false, false, true, false, false, true, true],
    [false, true, true, true, true, false, true],
    [false, true, false, false, false, true, true],
    [false, true, true, false, false, false, true],
    [false, true, false, true, true, true, true],
    [false, true, true, true, false, true, true],
    [false, true, true, false, true, true, true],
    [false, false, false, true, false, true, true],
];

/// Le premier chiffre ne se barre pas : il vit dans la **parité** des six suivants.
/// `true` = jeu L, `false` = jeu G. C'est ce qui permet à treize chiffres de tenir dans
/// douze groupes de barres, et c'est la partie de la norme qu'aucun œil ne relit.
const PARITE: [[bool; 6]; 10] = [
    [true, true, true, true, true, true],
    [true, true, false, true, false, false],
    [true, true, false, false, true, false],
    [true, true, false, false, false, true],
    [true, false, true, true, false, false],
    [true, false, false, true, true, false],
    [true, false, false, false, true, true],
    [true, false, true, false, true, false],
    [true, false, true, false, false, true],
    [true, false, false, true, false, true],
];

/// Un ISBN accepté, réduit aux treize chiffres qui se barrent.
///
/// La forme saisie ne survit pas ici — elle vit dans le `Livre`, avec ses tirets, parce
/// que c'est elle qui s'imprime en clair au-dessus du symbole. Les tirets d'un ISBN ne se
/// recalculent pas sans la table des préfixes d'éditeur, et l'application ne l'a pas.
#[derive(Debug, Clone, PartialEq)]
pub struct Isbn {
    chiffres: [u8; 13],
}

impl Isbn {
    /// Lit un ISBN saisi, sous l'une de ses deux formes, ou dit pourquoi il n'en est pas un.
    ///
    /// Le refus nomme ce qui cloche — longueur, préfixe ou clé —, parce que c'est la seule
    /// chose qui distingue un ISBN inventé d'un chiffre mal recopié, et que la correction
    /// n'est pas la même.
    pub fn lire(saisi: &str) -> Result<Self, String> {
        let net: Vec<char> = saisi
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
            .collect();
        match net.len() {
            10 => Self::depuis_dix(&net),
            13 => Self::depuis_treize(&net),
            n => Err(format!(
                "ISBN : {n} caractères une fois les tirets retirés, attendu 10 ou 13."
            )),
        }
    }

    /// Un ISBN-13, la forme d'aujourd'hui : treize chiffres et un préfixe de livre.
    fn depuis_treize(net: &[char]) -> Result<Self, String> {
        let mut chiffres = [0u8; 13];
        for (i, c) in net.iter().enumerate() {
            chiffres[i] = c
                .to_digit(10)
                .ok_or_else(|| format!("ISBN : « {c} » n'est pas un chiffre."))?
                as u8;
        }
        // Le préfixe avant la clé : « 3017620422003 » est un EAN-13 parfaitement valide, et
        // c'est un pot de pâte à tartiner. Sans ce contrôle, il irait s'imprimer en 4ème.
        let prefixe = &chiffres[..3];
        if prefixe != [9, 7, 8] && prefixe != [9, 7, 9] {
            return Err(format!(
                "ISBN : le préfixe {}{}{} n'est pas celui d'un livre — attendu 978 ou 979.",
                prefixe[0], prefixe[1], prefixe[2]
            ));
        }
        let attendue = cle(&chiffres[..12]);
        if attendue != chiffres[12] {
            return Err(format!(
                "ISBN : clé de contrôle {} au lieu de {attendue} — un chiffre a dû être mal \
                 recopié.",
                chiffres[12]
            ));
        }
        Ok(Self { chiffres })
    }

    /// Un ISBN-10, la forme d'avant 2007. Devient `978` + ses neuf premiers chiffres, avec
    /// une clé **recalculée** : celle de l'ISBN-10 est en modulo 11 et ne vaut plus rien.
    ///
    /// Sa clé est vérifiée **avant** la conversion, et c'est tout l'enjeu : sans ce
    /// contrôle, un ISBN-10 mal recopié produirait un EAN-13 valide et faux — la seule
    /// erreur de saisie qui pourrait traverser la chaîne entière sans jamais se voir.
    fn depuis_dix(net: &[char]) -> Result<Self, String> {
        let mut dix = [0u8; 10];
        for (i, c) in net.iter().enumerate() {
            // Le `X` vaut dix, et seulement en dernière position : c'est le seul reste du
            // modulo 11 que dix chiffres ne savent pas écrire.
            dix[i] = match (c, i) {
                ('X' | 'x', 9) => 10,
                _ => c
                    .to_digit(10)
                    .ok_or_else(|| format!("ISBN : « {c} » n'est pas un chiffre."))?
                    as u8,
            };
        }
        let somme: u32 = dix
            .iter()
            .enumerate()
            .map(|(i, d)| (10 - i as u32) * *d as u32)
            .sum();
        if !somme.is_multiple_of(11) {
            return Err(
                "ISBN à dix chiffres : la clé de contrôle ne ferme pas — un chiffre \
                        a dû être mal recopié."
                    .into(),
            );
        }
        let mut chiffres = [0u8; 13];
        chiffres[..3].copy_from_slice(&[9, 7, 8]);
        chiffres[3..12].copy_from_slice(&dix[..9]);
        chiffres[12] = cle(&chiffres[..12]);
        Ok(Self { chiffres })
    }

    /// Les treize chiffres, pour composer les groupes sous les barres.
    pub fn chiffres(&self) -> &[u8; 13] {
        &self.chiffres
    }

    /// Les 95 modules du symbole, `true` = barre.
    pub fn modules(&self) -> [bool; 95] {
        modules(&self.chiffres)
    }
}

/// La clé de contrôle des douze premiers chiffres : pondération 1, 3, 1, 3…
pub fn cle(douze: &[u8]) -> u8 {
    let somme: u32 = douze
        .iter()
        .enumerate()
        .map(|(i, d)| if i % 2 == 0 { *d as u32 } else { *d as u32 * 3 })
        .sum();
    ((10 - somme % 10) % 10) as u8
}

/// Les 95 modules d'un EAN-13 déjà validé.
///
/// Ne juge rien : la clé et le préfixe sont l'affaire de [`Isbn::lire`]. C'est ce partage
/// qui permet d'éprouver la structure du symbole sans passer par un ISBN valide.
pub fn modules(chiffres: &[u8; 13]) -> [bool; 95] {
    let mut m = [false; 95];
    let mut i = 0;
    let pose = |m: &mut [bool; 95], i: &mut usize, motif: &[bool]| {
        m[*i..*i + motif.len()].copy_from_slice(motif);
        *i += motif.len();
    };

    pose(&mut m, &mut i, &[true, false, true]);
    // Les six groupes de gauche, en L ou en G selon la parité que dicte le premier chiffre.
    // `G` est le miroir de `R`, lui-même complément de `L` : une seule table, deux
    // dérivations, et aucune troisième occasion de se tromper en la recopiant.
    for (g, d) in chiffres[1..7].iter().enumerate() {
        let l = L[*d as usize];
        let motif: Vec<bool> = if PARITE[chiffres[0] as usize][g] {
            l.to_vec()
        } else {
            l.iter().rev().map(|b| !b).collect()
        };
        pose(&mut m, &mut i, &motif);
    }
    pose(&mut m, &mut i, &[false, true, false, true, false]);
    // La moitié droite est toujours en R, le complément de L — d'où ses groupes de parité
    // paire, qui disent au lecteur optique par quel bout il tient le code.
    for d in &chiffres[7..13] {
        let motif: Vec<bool> = L[*d as usize].iter().map(|b| !b).collect();
        pose(&mut m, &mut i, &motif);
    }
    pose(&mut m, &mut i, &[true, false, true]);
    debug_assert_eq!(i, 95);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les deux tables de la norme, recopiées **sous une autre forme** que celle du code :
    /// lui les porte en `[bool; 7]` et `[bool; 6]`, le test en chaînes.
    ///
    /// Une table qui se compare à elle-même ne protège rien, et ce n'est pas une crainte
    /// théorique : la première version de ce fichier lisait `PARITE` pour vérifier `PARITE`,
    /// et une mutation posée dessus — un `false` retourné en `true` — laissait les quatorze
    /// tests au vert. Le symbole se serait scanné parfaitement et aurait rendu un autre
    /// livre. Recopier la norme dans une autre écriture est ce qui rend les deux fautes
    /// indépendantes.
    const L_NORME: [&str; 10] = [
        "0001101", "0011001", "0010011", "0111101", "0100011", "0110001", "0101111", "0111011",
        "0110111", "0001011",
    ];
    const PARITE_NORME: [&str; 10] = [
        "LLLLLL", "LLGLGG", "LLGGLG", "LLGGGL", "LGLLGG", "LGGLLG", "LGGGLL", "LGLGLG", "LGLGGL",
        "LGGLGL",
    ];

    #[test]
    fn les_tables_sont_celles_de_la_norme() {
        for (d, attendu) in L_NORME.iter().enumerate() {
            let lu: String = L[d].iter().map(|b| if *b { '1' } else { '0' }).collect();
            assert_eq!(&lu, attendu, "jeu L du chiffre {d}");
        }
        for (d, attendu) in PARITE_NORME.iter().enumerate() {
            let lu: String = PARITE[d]
                .iter()
                .map(|b| if *b { 'L' } else { 'G' })
                .collect();
            assert_eq!(&lu, attendu, "parité du premier chiffre {d}");
        }
    }

    /// Les treize chiffres d'un ISBN écrit tel qu'on le lit sur un livre.
    fn c(s: &str) -> [u8; 13] {
        let v: Vec<u8> = s
            .bytes()
            .filter(|b| b.is_ascii_digit())
            .map(|b| b - b'0')
            .collect();
        v.try_into().expect("treize chiffres")
    }

    /* ---------- la clé ---------- */

    /// Trois ISBN réels, recopiés de leur page de copyright. La clé est le seul garde-fou
    /// contre un chiffre mal recopié, et c'est exactement l'erreur qu'on fait en saisissant.
    #[test]
    fn la_cle_ferme_un_isbn_reel() {
        for isbn in ["9780306406157", "9782070413119", "9791030201048"] {
            let d = c(isbn);
            assert_eq!(cle(&d[..12]), d[12], "{isbn}");
        }
    }

    /// Un seul chiffre changé doit rompre la clé. Sans ce test, une clé qui rendrait
    /// toujours le dernier chiffre passerait le test précédent.
    #[test]
    fn un_chiffre_change_rompt_la_cle() {
        let bon = c("9780306406157");
        for i in 0..12 {
            let mut faux = bon;
            faux[i] = (faux[i] + 1) % 10;
            assert_ne!(
                cle(&faux[..12]),
                faux[12],
                "chiffre {i} modifié, clé inchangée"
            );
        }
    }

    /* ---------- ce que `lire` accepte et refuse ---------- */

    #[test]
    fn un_isbn_13_valide_se_lit() {
        assert_eq!(
            Isbn::lire("978-2-07-041311-9").unwrap().chiffres(),
            &c("9782070413119")
        );
    }

    /// Les tirets sont une commodité de lecture, et leur place varie avec l'éditeur.
    #[test]
    fn les_tirets_et_les_espaces_ne_comptent_pas() {
        let sans = Isbn::lire("9782070413119").unwrap();
        for forme in [
            "978-2-07-041311-9",
            "978 2 07 041311 9",
            "  978-2070413119 ",
        ] {
            assert_eq!(Isbn::lire(forme).unwrap(), sans, "{forme}");
        }
    }

    /// L'exemple canonique de la norme : un ISBN-10 devient `978` + ses neuf premiers,
    /// suivi d'une clé **recalculée** — celle de l'ISBN-10 ne vaut plus rien.
    #[test]
    fn un_isbn_10_se_convertit() {
        assert_eq!(
            Isbn::lire("0-306-40615-2").unwrap().chiffres(),
            &c("9780306406157")
        );
    }

    /// Le `X` terminal d'un ISBN-10 vaut dix, et seulement en dernière position. Sa clé
    /// est en modulo 11, seul reste possible que dix chiffres ne savent pas écrire.
    #[test]
    fn le_x_terminal_vaut_dix() {
        assert!(
            Isbn::lire("0-8044-2957-X").is_ok(),
            "X refusé sur un ISBN-10 valide"
        );
        assert!(
            Isbn::lire("0-8044-295X-7").is_err(),
            "X accepté ailleurs qu'en fin"
        );
    }

    /// **La clé de l'ISBN-10 se vérifie avant la conversion.** Sans ce contrôle, un
    /// ISBN-10 mal recopié produirait un EAN-13 parfaitement valide — et faux. C'est le
    /// seul endroit où une erreur de saisie pourrait traverser toute la chaîne sans bruit.
    #[test]
    fn un_isbn_10_de_cle_fausse_est_refuse_avant_conversion() {
        assert!(
            Isbn::lire("0-306-40615-3").is_err(),
            "clé ISBN-10 fausse acceptée"
        );
    }

    #[test]
    fn une_cle_13_fausse_est_refusee() {
        assert!(Isbn::lire("9782070413118").is_err());
    }

    /// Un ISBN commence par 978 ou 979. Un EAN-13 d'épicerie a une clé valide et n'est
    /// pas un ISBN : le refuser est ce qui empêche un code de yaourt d'aller en 4ème.
    #[test]
    fn un_prefixe_qui_n_est_pas_un_livre_est_refuse() {
        assert!(
            Isbn::lire("3017620422003").is_err(),
            "préfixe non-livre accepté"
        );
    }

    #[test]
    fn une_longueur_impossible_est_refusee() {
        for saisi in ["", "978", "97820704131190", "abcdefghijklm"] {
            assert!(Isbn::lire(saisi).is_err(), "« {saisi} » accepté");
        }
    }

    /* ---------- le symbole ---------- */

    #[test]
    fn les_gardes_sont_a_leur_place() {
        let m = modules(&c("9782070413119"));
        assert_eq!(&m[0..3], [true, false, true], "garde gauche");
        assert_eq!(
            &m[45..50],
            [false, true, false, true, false],
            "garde centrale"
        );
        assert_eq!(&m[92..95], [true, false, true], "garde droite");
    }

    /// **Le test qui porte le module.** Le premier chiffre n'est pas barré : il se lit à la
    /// parité des six groupes de gauche — un nombre impair de modules noirs pour L, pair
    /// pour G. Une table de parité fausse produit un symbole qui se scanne parfaitement et
    /// rend le mauvais livre. Rien, ni un aperçu ni une relecture, ne peut le voir.
    #[test]
    fn le_premier_chiffre_se_lit_a_la_parite_des_six_groupes() {
        for premier in 0..10u8 {
            let mut d = [0u8; 13];
            d[0] = premier;
            for (i, x) in d.iter_mut().enumerate().skip(1) {
                *x = (i as u8) % 10;
            }
            let m = modules(&d);
            let lu: Vec<bool> = (0..6)
                .map(|g| m[3 + g * 7..3 + g * 7 + 7].iter().filter(|b| **b).count() % 2 == 1)
                .collect();
            let lu: String = lu.iter().map(|b| if *b { 'L' } else { 'G' }).collect();
            assert_eq!(
                lu, PARITE_NORME[premier as usize],
                "premier chiffre {premier}"
            );
        }
    }

    /// La moitié droite est toujours en jeu R, complément de L : ses groupes portent donc
    /// un nombre **pair** de modules noirs, quel que soit le chiffre.
    #[test]
    fn la_moitie_droite_est_toujours_de_parite_paire() {
        let m = modules(&c("9780306406157"));
        for g in 0..6 {
            let n = m[50 + g * 7..50 + g * 7 + 7].iter().filter(|b| **b).count();
            assert_eq!(n % 2, 0, "groupe droit {g} de parité impaire");
        }
    }

    /// Un chiffre du milieu ne doit toucher que les sept modules de son groupe. Éprouvé
    /// sur des chiffres bruts et non sur deux ISBN : changer un chiffre d'un ISBN change
    /// aussi sa clé, donc deux groupes — le test ne verrait plus ce qu'il cherche.
    ///
    /// Le premier chiffre fait exception et a son propre test : lui gouverne la parité des
    /// six groupes de gauche, et c'est précisément son travail de les toucher tous.
    #[test]
    fn un_chiffre_ne_touche_que_son_groupe() {
        let base = c("9780306406157");
        for i in 1..13 {
            let mut autre = base;
            autre[i] = (autre[i] + 1) % 10;
            let (a, b) = (modules(&base), modules(&autre));
            let bouge: Vec<usize> = (0..95).filter(|k| a[*k] != b[*k]).collect();
            // Les six premiers groupes suivent la garde gauche, les six autres la centrale.
            let debut = if i <= 6 {
                3 + (i - 1) * 7
            } else {
                50 + (i - 7) * 7
            };
            assert!(
                bouge.iter().all(|k| (debut..debut + 7).contains(k)),
                "le chiffre {i} déborde de son groupe : {bouge:?}"
            );
        }
    }
}
