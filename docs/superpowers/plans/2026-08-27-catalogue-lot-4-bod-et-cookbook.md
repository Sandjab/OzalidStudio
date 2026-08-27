# Catalogue lot 4 — BoD complété, le COOKBOOK, et les dettes closes

> **Pour les agents :** SOUS-SKILL REQUIS — `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont cochables (`- [ ]`).

**But :** faire descendre dans le catalogue tout ce que BoD publie — dix formats, quatre
papiers, trois finitions —, étendre le schéma au seul endroit où le relevé le contredit,
puis fermer le chantier : le COOKBOOK réécrit pour le monde en fichiers, le mot
« prestataire » retiré, et les deux dettes de fond du lot 3 closes.

**Architecture :** le schéma gagne **un** champ, `Papier.pages`, parce que BoD plafonne le
photo brillant 130 g à 868 pages là où sa reliure va à 900 — une contrainte du papier que
`Reliure.pages` ne sait pas dire. L'intersection se calcule là où la pagination se contrôle
déjà, dans `package.rs`, extraite en fonction pure pour être testable. Le reste du lot est
de la donnée (`bod.toml`), de la documentation (`COOKBOOK.md`), et deux corrections de
silence : `envois.js` cesse de lire les papiers dans la table plate, `Livraison::normalise`
cesse de retirer un livrable sans un mot.

**Pile :** Rust (Tauri 2, serde, toml), front vanilla sans bundler, tests `cargo test` et
`node --test`.

---

## Décisions arbitrées (utilisateur, 27/08) — ne pas les rouvrir

1. **Le schéma s'étend** d'un `pages` optionnel sur le papier. Les trois autres issues —
   renoncer au quatrième papier, plafonner tout le monde à 868, ou inscrire 900 et le
   signaler au cookbook — inscrivaient une valeur fausse ou un silence.
2. **Le mot « prestataire » est pris dans ce lot**, cookbook compris — titre et chapitre.
3. **Les deux dettes de fond du lot 3 sont prises** : `envois.js` et `Livraison::normalise`.

Les valeurs relevées chez BoD, leur provenance et le dispositif pour les rejouer sont dans
`docs/superpowers/2026-08-26-reconnaissance-lot-4.md`, deuxième partie. **Aucune valeur de
ce plan ne doit être recalculée ou « corrigée » à l'exécution** : elles ont été relevées, et
une valeur devinée est précisément ce que le chantier interdit.

## Invariants sur lesquels ce plan s'appuie

- **Le témoin ne bouge pas : 98 pages, dos 7,21 mm.** `examples/temoin.rs:26` nomme sa
  fabrication en dur — `("bod", "135x215", "broche", "creme-90")` —, il ne dépend donc pas
  de l'ordre du fichier. Mais il dépend des **marges** du format `135x215`, qui ne changent
  pas d'un iota (tâche 2, piège 1).
- **`Pod::verifie` refuse au chargement** un fichier qui promet ce que le code ne tient
  pas : clé mal formée, doublon, marge qui déborde, tranches de gouttière qui se recouvrent,
  POD sans reliure composable. Un `bod.toml` mal écrit ne passera pas en silence.
- **`Pod::fabrication_defaut`** prend le premier format, la première reliure composable et
  le premier papier ; `aplatit` met cette entrée en tête de la table plate. L'ordre
  d'écriture du fichier est donc un choix d'interface.
- **La pagination admise vit sur la reliure** (`Reliure.pages`), et `aplatit` la recopie
  dans `Provider.pages_min/max`. La tâche 1 ajoute une seconde source de restriction sans
  déplacer la première.

## Avant chaque commit

Dans l'ordre, tous obligatoires :

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd .. && node --test tests/*.test.js
```

Et, dès qu'un fichier de `src-tauri/` a changé :

```bash
cd src-tauri && cargo run --example temoin
```

Attendu, à chaque fois : **98 pages, dos 7,21 mm**. Un écart n'est pas à « expliquer » : il
arrête la tâche.

Après un changement de `src/` seul : `touch src-tauri/src/lib.rs` avant `cargo build`, sinon
le binaire garde l'ancien front.

**Tout test neuf doit avoir été vu échouer.** Chaque tâche ci-dessous a une étape « lancer
le test et le voir échouer » avec le message attendu ; sauter cette étape rend le test
inutile — voir la mémoire `tests-qui-ne-protegent-rien`.

## Pièges transverses

1. **`135x215` reste le premier format du fichier et garde ses valeurs exactes** — marge
   haute 18,8 et non 18,75, que le modèle Word donne pourtant. C'est un arrondi assumé du
   lot 1 ; le « corriger » changerait la hauteur du bloc, donc la pagination, donc le
   témoin. La tâche 2 pose un test qui l'ancre.
2. **`deny_unknown_fields` est actif sur `Papier`** : le champ `pages` doit être ajouté à la
   structure Rust **avant** d'apparaître dans un `.toml`, sinon le fichier est refusé au
   chargement et les six PODs disparaissent d'un coup. Tâche 1 avant tâche 2, sans
   exception.
3. **Les fixtures de test JS posent `papiers` sur des providers plats** (`coquille.test.js`,
   `composition.test.js`, `contrats.test.js`). La tâche 4 retire ce champ de `ProviderVue` :
   les fixtures doivent suivre, sinon elles décrivent une vue qui n'existe plus.
4. **Le renommage de la tâche 6 touche beaucoup de lignes.** Le faire en dernier avant la
   documentation évite de renommer du code que les tâches 1 à 5 réécrivent.

## Structure des fichiers

| Fichier | Rôle dans ce lot |
|---|---|
| `src-tauri/src/catalogue.rs` | `Papier.pages`, sa validation, `Papier::bornes_dans` |
| `src-tauri/src/package.rs` | `verifie_pagination` extraite et testée, appel de `bornes_dans` |
| `src-tauri/pods/bod.toml` | dix formats, quatre papiers, trois finitions, raison réécrite |
| `src-tauri/src/commands.rs` | `ProviderVue.papiers` retiré ; test de fixture `dos_publie` |
| `src-tauri/src/projet.rs` | `normalise` rend ce qu'il élague |
| `src/envois.js` | la teinte vient de l'arbre |
| `src/app.js`, `src/livraison.js` | les élagués s'affichent |
| `docs/COOKBOOK.md` | pointeurs morts, chapitre d'ajout, chapitre BoD |
| `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` | lot 4 coché |

---

### Tâche 1 : Un papier peut plafonner la pagination

**Fichiers :**
- Modifier : `src-tauri/src/catalogue.rs` (`Papier` ~l. 182-197, `verifie_papier` ~l. 526)
- Modifier : `src-tauri/src/package.rs:166-171`
- Test : `src-tauri/src/catalogue.rs` (module `tests`), `src-tauri/src/package.rs` (module `tests`)

- [ ] **Étape 1 : écrire le test du refus, dans `package.rs`, module `tests`**

Le contrôle de pagination vit aujourd'hui inline dans `assemble`, qu'aucun test ne peut
appeler sans Typst. On l'extrait, comme `verifie_pages` l'est déjà dans le même fichier.

```rust
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
```

Et les deux fabriques, dans le même module de tests :

```rust
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
```

`Papier` est importé directement dans `package.rs` — pas `catalogue::Papier`. La fabrique
`provider_d_essai()` existe déjà (vers la l. 622) et construit un `Papier` littéral :
**l'étape 3 la casse**, il faut y ajouter `pages: None`. Le compilateur le dira ; ne pas en
profiter pour lui donner un plafond, elle sert d'autres tests.

- [ ] **Étape 2 : lancer le test et le voir échouer**

```bash
cd src-tauri && cargo test le_plafond_du_papier_resserre_celui_de_la_reliure
```

Attendu : **échec de compilation** — `verifie_pagination` n'existe pas, et `Papier` n'a pas
de champ `pages`. C'est le rouge qui compte : il dit que ni la fonction ni le champ n'étaient
là.

- [ ] **Étape 3 : ajouter le champ au schéma**

Dans `catalogue.rs`, structure `Papier` — après `dos`, avant `source` :

```rust
    /// Pagination que **ce papier** admet, quand il est plus restrictif que la reliure.
    ///
    /// Absent chez le cas courant : c'est la reliure qui borne, et un papier n'a rien à
    /// redire. Présent chez BoD, dont le photo brillant 130 g plafonne à 868 pages là où
    /// le broché va à 900 — une contrainte de l'épaisseur du papier, pas du collage.
    ///
    /// Les deux bornes se **croisent**, elles ne se remplacent pas : le livrable admet ce
    /// que la reliure et le papier admettent tous deux.
    #[serde(default)]
    pub pages: Option<Pagination>,
```

- [ ] **Étape 4 : valider le champ à la lecture**

Dans `verifie_papier`, après le contrôle du dos :

```rust
        if let Some(p) = &pa.pages {
            if p.min == 0 || p.min > p.max {
                return Err(format!(
                    "{ou} : pagination de papier impossible ({} à {}).",
                    p.min, p.max
                ));
            }
        }
```

- [ ] **Étape 5 : croiser les bornes, sur le papier**

Dans `catalogue.rs`, après la structure `Papier` :

```rust
impl Papier {
    /// Les bornes de pagination d'un livrable fait de ce papier, à l'intérieur de celles
    /// que la reliure impose. Sans plafond propre, le papier ne resserre rien.
    pub fn bornes_dans(&self, min: u32, max: u32) -> (u32, u32) {
        match self.pages {
            Some(p) => (min.max(p.min), max.min(p.max)),
            None => (min, max),
        }
    }
}
```

- [ ] **Étape 6 : extraire le contrôle de `package.rs` en fonction pure**

Remplacer `package.rs:166-171` par un appel, et poser la fonction juste avant `assemble` :

```rust
/// Refuse une pagination hors de ce que le livrable admet, en nommant qui la borne.
///
/// Hors d'`assemble` pour être testable : le contrôle y était inline, et aucun test ne
/// pouvait l'atteindre sans composer un intérieur. C'est le même arbitrage que
/// `verifie_pages`, dans ce fichier.
///
/// Le message nomme le **papier** quand c'est lui qui resserre : « hors des 24 à 900 que
/// BoD accepte en broche » enverrait chercher l'erreur du mauvais côté pour un livre de
/// 880 pages en photo brillant.
fn verifie_pagination(cle: &str, pages: u32, pr: &Provider, papier: &Papier) -> Result<(), String> {
    let (min, max) = papier.bornes_dans(pr.pages_min, pr.pages_max);
    if pages >= min && pages <= max {
        return Ok(());
    }
    let en = if (min, max) == (pr.pages_min, pr.pages_max) {
        pr.fabrication.reliure.clone()
    } else {
        format!("{} en {}", pr.fabrication.reliure, papier.nom)
    };
    Err(format!(
        "{cle} : {pages} pages, hors des {min} à {max} que {} accepte en {en}.",
        pr.libelle
    ))
}
```

À l'emplacement du contrôle supprimé, en gardant le commentaire qui explique que le refus
tombe après la composition :

```rust
    // Ce contrôle tombe après la composition de l'intérieur du gabarit : un refus ici
    // coûte donc une composition — que la mémoïsation de `lot` ne repaie pas pour le
    // livrable suivant du même gabarit, qui la retentera à son tour.
    verifie_pagination(cle, interieur.pages, pr, papier)?;
```

- [ ] **Étape 7 : lancer le test et le voir passer**

```bash
cd src-tauri && cargo test le_plafond_du_papier_resserre_celui_de_la_reliure
```

Attendu : **PASS**.

- [ ] **Étape 8 : le test de validation du champ**

Dans le module de tests de `catalogue.rs`, à côté des autres refus de fichier :

```rust
    /// Une pagination de papier inversée est refusée au chargement, comme les autres
    /// valeurs impossibles : un fichier qui l'écrit se corrige, il ne se devine pas.
    #[test]
    fn une_pagination_de_papier_inversee_est_refusee() {
        let toml = r#"
cle = "essai"
nom = "Essai"
[[format]]
cle = "a"
nom = "A"
mm = { largeur = 100.0, hauteur = 150.0 }
marges = { haut = 10.0, bas = 10.0, exterieur = 10.0 }
gouttieres = [ { de = 24, a = 100, mm = 15.0 } ]
[[reliure]]
cle = "broche"
nom = "Broché"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 100 }
parite = "paire"
[[papier]]
cle = "p"
nom = "P"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.06, plus = 0.0 }
pages = { min = 200, max = 100 }
"#;
        let err = Pod::depuis_toml(toml).unwrap_err();
        assert!(err.contains("pagination de papier impossible"), "{err}");
    }
```

- [ ] **Étape 9 : le voir échouer, puis passer**

```bash
cd src-tauri && cargo test une_pagination_de_papier_inversee_est_refusee
```

Si l'étape 4 est déjà écrite, ce test passe du premier coup — **c'est un test qui n'a jamais
été rouge**. Le rendre rouge par mutation ciblée : commenter le `return Err` de l'étape 4,
relancer, voir l'échec, décommenter, relancer. Sans cette mutation, le test ne protège rien.

- [ ] **Étape 10 : vérifications complètes et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo run --example temoin   # 98 pages, dos 7,21 mm
cd .. && node --test tests/*.test.js
git add src-tauri/src/catalogue.rs src-tauri/src/package.rs
git commit -m "Un papier peut dire jusqu'où il va, et le refus le nomme"
```

---

### Tâche 2 : BoD porte tout ce qu'il publie

**Fichiers :**
- Modifier : `src-tauri/pods/bod.toml` (réécriture complète)
- Test : `src-tauri/src/catalogue.rs` (module `tests`)

- [ ] **Étape 1 : écrire le test qui ancre le format historique**

C'est le garde-fou du témoin. Dans le module de tests de `catalogue.rs` :

```rust
    /// Le format historique de BoD ne bouge pas d'un dixième.
    ///
    /// Le modèle Word « Roman A » donne 18,75 mm en marge haute ; la table porte 18,8
    /// depuis le lot 1, arrondi assumé. Le relevé du lot 4 a confirmé la source sans
    /// autoriser la correction : reprendre 18,75 changerait la hauteur du bloc de texte,
    /// donc la pagination, donc le dos — et `cargo run --example temoin` cesserait de
    /// valoir 98 pages sans que rien ne dise pourquoi.
    #[test]
    fn le_format_historique_de_bod_ne_bouge_pas() {
        let bod = pod("bod").expect("BoD est fourni");
        let f = bod
            .formats
            .iter()
            .find(|f| f.cle == "135x215")
            .expect("BoD garde son format historique");
        assert_eq!((f.mm.largeur, f.mm.hauteur), (135.0, 215.0));
        assert_eq!(f.marges.haut, 18.8, "l'arrondi du lot 1 fait foi");
        assert_eq!(f.marges.bas, 28.0);
        assert_eq!(f.marges.exterieur, 15.0);
        assert_eq!(f.gouttieres.len(), 1);
        assert_eq!((f.gouttieres[0].de, f.gouttieres[0].a), (24, 900));
        assert_eq!(f.gouttieres[0].mm, 20.0);
        assert_eq!(
            bod.formats.first().map(|f| f.cle.as_str()),
            Some("135x215"),
            "il reste en tête : c'est lui que la cascade propose d'office"
        );
    }
```

- [ ] **Étape 2 : lancer le test et le voir passer, puis le rendre rouge par mutation**

```bash
cd src-tauri && cargo test le_format_historique_de_bod_ne_bouge_pas
```

Il passe sur le fichier actuel — c'est normal, il décrit l'existant. Le rendre rouge une
fois : dans `bod.toml`, écrire `haut = 18.75`, relancer, voir l'échec nommer 18,8, puis
remettre `18.8`. **Sans cette mutation, ce test n'a jamais rien prouvé.**

- [ ] **Étape 3 : réécrire `src-tauri/pods/bod.toml`**

Contenu complet. Les valeurs viennent de la reconnaissance, §§ 6 à 8 — ne rien recalculer.

```toml
# BoD (Books on Demand) — Hambourg, filiale française, impression Europe.
#
# Imprimer n'oblige pas à publier : le parcours myBoD permet de commander pour soi sans
# référencer le titre. C'est ce qui en fait le défaut du comparatif POD du 19/08/2026.
#
# Formats et marges relevés dans les modèles Word « Roman A » du 11/12/2024, épaisseurs et
# bornes au calculateur de couverture officiel
# (https://www.bod.fr/aide/calcul-de-la-couverture.html), le 26/08/2026. Les quatre modèles
# de BoD ne s'accordent pas partout — Roman A fait foi, c'est celui dont le format
# historique est tiré. Le détail des divergences est dans la reconnaissance du lot 4,
# verdict 3.

cle = "bod"
nom = "BoD"
# Guide de maquette BoD, confirmé par le calculateur, qui rend « fond perdu » à 0,5 cm sur
# tous les formats interrogés. Commun à ses formats.
fond_perdu = 5.0

# Le format historique reste en tête : c'est lui que la cascade propose d'office, et le
# seul sur lequel des livres ont déjà été composés. Ses marges portent l'arrondi du lot 1
# — 18,8 là où le modèle Word donne 18,75 — et ne doivent pas être « corrigées » : le
# témoin de non-régression en dépend.
[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
mm = { largeur = 135.0, hauteur = 215.0 }
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
# BoD ne module pas la marge de reliure selon l'épaisseur — tranche unique, couvrant les
# 24 à 900 pages que sa couverture souple admet.
gouttieres = [ { de = 24, a = 900, mm = 20.0 } ]
source = "modèle Word « Roman A » 13,5 × 21,5"

[[format]]
cle = "120x190"
nom = "12 × 19 cm"
mm = { largeur = 120.0, hauteur = 190.0 }
marges = { haut = 15.0, bas = 22.0, exterieur = 15.0 }
gouttieres = [ { de = 24, a = 900, mm = 18.0 } ]
source = "modèle Word « Roman A » 12 × 19"

[[format]]
cle = "148x210"
nom = "14,8 × 21 cm"
mm = { largeur = 148.0, hauteur = 210.0 }
marges = { haut = 18.7, bas = 28.0, exterieur = 16.0 }
gouttieres = [ { de = 24, a = 900, mm = 22.0 } ]
source = "modèle Word « Roman A » 14,8 × 21"

[[format]]
cle = "155x220"
nom = "15,5 × 22 cm"
mm = { largeur = 155.0, hauteur = 220.0 }
marges = { haut = 18.7, bas = 28.0, exterieur = 16.0 }
gouttieres = [ { de = 24, a = 900, mm = 22.0 } ]
source = "modèle Word « Roman A » 15,5 × 22"

[[format]]
cle = "170x170"
nom = "17 × 17 cm"
mm = { largeur = 170.0, hauteur = 170.0 }
marges = { haut = 13.0, bas = 24.0, exterieur = 16.0 }
gouttieres = [ { de = 24, a = 900, mm = 21.0 } ]
source = "modèle Word « Roman A » 17 × 17"

[[format]]
cle = "170x220"
nom = "17 × 22 cm"
mm = { largeur = 170.0, hauteur = 220.0 }
marges = { haut = 18.7, bas = 28.0, exterieur = 16.0 }
gouttieres = [ { de = 24, a = 900, mm = 22.0 } ]
source = "modèle Word « Roman A » 17 × 22"

[[format]]
cle = "190x270"
nom = "19 × 27 cm"
mm = { largeur = 190.0, hauteur = 270.0 }
marges = { haut = 21.0, bas = 28.0, exterieur = 22.0 }
gouttieres = [ { de = 24, a = 900, mm = 26.0 } ]
source = "modèle Word « Roman A » 19 × 27"

# BoD nomme ce format « 21 x 15 cm » : c'est le seul à l'italienne de son catalogue. Son
# modèle « Livre pratique A » du même nom contient une page de 210 × 210 mm — un fichier
# mal préparé chez eux ; les trois autres modèles s'accordent sur 210 × 150.
[[format]]
cle = "210x150"
nom = "21 × 15 cm"
mm = { largeur = 210.0, hauteur = 150.0 }
marges = { haut = 14.0, bas = 23.0, exterieur = 16.0 }
gouttieres = [ { de = 24, a = 900, mm = 21.0 } ]
source = "modèle Word « Roman A » 21 × 15 ; le « Livre pratique A » du même nom est faux"

[[format]]
cle = "210x210"
nom = "21 × 21 cm"
mm = { largeur = 210.0, hauteur = 210.0 }
marges = { haut = 16.0, bas = 26.0, exterieur = 19.0 }
gouttieres = [ { de = 24, a = 900, mm = 24.0 } ]
source = "modèle Word « Roman A » 21 × 21"

[[format]]
cle = "210x297"
nom = "21 × 29,7 cm"
mm = { largeur = 210.0, hauteur = 297.0 }
marges = { haut = 24.0, bas = 33.0, exterieur = 22.3 }
gouttieres = [ { de = 24, a = 900, mm = 26.0 } ]
source = "modèle Word « Roman A » 21 × 29,7 ; le « Roman B » donne 14 mm en haut, écarté"

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }
parite = "paire"
source = "calculateur : minimum 24 pages, maximum 900, compte pair obligatoire"

# La géométrie est relevable — le calculateur rend le rempli et le rabat —, mais `planche`
# ne sait pas la composer : une couverture rigide déborde du livre, se replie à l'intérieur
# des plats et se monte sur des cartons. La spec range ce chantier hors périmètre.
[[reliure]]
cle = "rigide"
nom = "Couverture rigide"
non_outille = "planche ne sait pas composer une couverture rigide : ni rempli, ni mors, ni cartons"

# Trois pelliculages, offerts sur toutes les couvertures. Aucun n'a d'effet sur la
# composition : la finition nomme une option de commande, et le livrable en garde la trace.
[[finition]]
cle = "mat"
nom = "Pelliculage mat"

[[finition]]
cle = "brillant"
nom = "Pelliculage brillant"

[[finition]]
cle = "relief"
nom = "Pelliculage en relief"

# BoD : dos = pages × épaisseur_feuille/2 + 0,6 mm de couverture. Le terme constant est le
# carton de 250 g, identique pour les quatre papiers — vérifié sur chaque relevé, où le
# calculateur sépare l'épaisseur du bloc (`spine_width`) du dos total (`thickness`).
# Le crème 90 g reste en tête : c'est le papier d'office de BoD, et celui du roman.
[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { forme = "multiplie", par = 0.0675, plus = 0.6 }
source = "calculateur, 0,0135 cm/feuille — 24 p → 1,62 mm, 280 p → 19,5 mm, 868 p → 58,59 mm"

[[papier]]
cle = "blanc-90"
nom = "Blanc 90 g"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.06, plus = 0.6 }
source = "calculateur, 0,012 cm/feuille — 24 p → 1,44 mm, 868 p → 52,08 mm"

[[papier]]
cle = "photo-mat-120"
nom = "Photo mat 120 g"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.063, plus = 0.6 }
source = "calculateur, 0,0126 cm/feuille — 24 p → 1,512 mm, 500 p → 31,5 mm"

# Le seul papier qui plafonne plus bas que sa reliure : 868 pages contre 900. La valeur
# vient d'une clé de configuration du calculateur (`[PhotoBrilliant]: 868`, relevé du lot 4,
# § 8), sans raison donnée avec la valeur. Ce n'est pas une question d'épaisseur : à 0,0101
# cm/feuille, c'est au contraire le plus mince des quatre papiers de la table.
[[papier]]
cle = "photo-brillant-130"
nom = "Photo brillant 130 g"
teinte = "#ffffff"
dos = { forme = "multiplie", par = 0.0505, plus = 0.6 }
pages = { min = 24, max = 868 }
source = "calculateur, 0,0101 cm/feuille — 24 p → 1,212 mm, 868 p → 43,834 mm ; plafond à 868"
```

- [ ] **Étape 4 : le test qui ancre le plafond du brillant sur le catalogue livré**

```rust
    /// Le seul papier du catalogue livré qui plafonne plus bas que sa reliure.
    ///
    /// Il donne au champ `pages` du papier son unique emploi réel, et c'est ce qui rend
    /// vérifiable, sur le catalogue fourni et non sur une fixture, que le croisement des
    /// bornes descend jusqu'au fichier.
    #[test]
    fn le_photo_brillant_de_bod_plafonne_plus_bas_que_sa_reliure() {
        let bod = pod("bod").expect("BoD est fourni");
        let broche = bod
            .reliures
            .iter()
            .find(|r| r.cle == "broche")
            .expect("BoD relie en broché");
        let brillant = bod
            .papiers
            .iter()
            .find(|p| p.cle == "photo-brillant-130")
            .expect("BoD offre le photo brillant");
        let pages = broche.pages.expect("une reliure composable porte sa pagination");

        assert_eq!(pages.max, 900);
        assert_eq!(brillant.bornes_dans(pages.min, pages.max), (24, 868));

        let creme = bod
            .papiers
            .iter()
            .find(|p| p.cle == "creme-90")
            .expect("BoD offre le crème");
        assert_eq!(
            creme.bornes_dans(pages.min, pages.max),
            (24, 900),
            "un papier sans plafond ne resserre rien"
        );
    }
```

- [ ] **Étape 5 : lancer, voir échouer si le fichier n'est pas encore écrit, puis passer**

```bash
cd src-tauri && cargo test bod
```

Attendu : les trois tests de BoD passent. Si `les_six_fichiers_fournis_se_lisent` échoue, le
message nomme la ligne fautive du TOML — le corriger sur le fichier, jamais sur le test.

- [ ] **Étape 6 : vérifications complètes**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo run --example temoin
```

Attendu : **98 pages, dos 7,21 mm**. Le témoin compose sur `135x215` en crème : dix formats
de plus ne changent rien, et c'est exactement ce que cette étape prouve.

```bash
cd .. && node --test tests/*.test.js
```

- [ ] **Étape 7 : commit**

```bash
git add src-tauri/pods/bod.toml src-tauri/src/catalogue.rs
git commit -m "BoD porte ses dix formats, ses quatre papiers et ses trois pelliculages"
```

---

### Tâche 3 : sans objet — la dette était déjà soldée

**Constaté le 27/08, avant toute modification. Rien à faire, rien n'a été fait.**

Cette tâche demandait d'écrire une fixture mêlant un papier à formule et un papier sans
formule, pour rendre protecteur un test que le lot 3 disait creux. **Cette fixture existe
depuis le lot 3** : `la_conversion_d_un_papier_suit_sa_propre_formule_de_dos`
(`commands.rs:2663`), posée par le commit « Quatre corrections de revue sur l'arbre du
catalogue ». Elle mêle les deux formes dans un même POD, et passe par `PodVue::from` — le
site d'appel réel, celui qu'une régression toucherait — là où cette tâche prescrivait
`PapierVue::from` en direct. Elle est donc au moins équivalente, et sur un point meilleure.

Deux erreurs, toutes deux dans la reconnaissance et le plan, aucune dans le code :

1. **La reconnaissance a été menée sur la mémoire plutôt que sur la source.** La mémoire du
   lot 3 nommait pourtant ce test ; elle a été lue comme « il faudrait un tel test » au lieu
   de « ce test existe et porte seul la règle ». Le verdict 6 de la reconnaissance a été
   corrigé en conséquence.
2. **La mutation prescrite ici ne démontrait pas ce qu'elle prétendait.** Remplacer
   `pa.dos.publie()` par `true` fait rougir **les deux** tests, puisque CoolLibri porte de
   vrais papiers sans formule sur le catalogue livré. La mutation qui les distinguerait
   devrait porter sur `PodVue::from`, seul à voir le POD.

Et une nuance sur le mot « creux » : `dos_publie_est_porte_par_chaque_papier` ancre ce que
le catalogue **livré** porte, ce qui a sa valeur propre. Ce qu'il ne peut pas protéger, c'est
la règle de portage — d'où l'autre test. Les deux restent.

---

### Tâche 4 : La teinte du canevas vient de l'arbre

**Fichiers :**
- Modifier : `src/envois.js:358-373`
- Modifier : `src-tauri/src/commands.rs` (`ProviderVue`, retrait de `papiers`)
- Modifier : `tests/coquille.test.js`, `tests/composition.test.js`, `tests/contrats.test.js` (fixtures)

`envois.js` est le dernier lecteur des papiers dans la table plate, avec un repli
`?? pr?.papiers[0]` — le motif même que le lot 3 a supprimé pour `dos_publie`. `app.js`
porte déjà `papierCourant()`, qui lit l'arbre et que le scope global rend accessible :
`envois.js` est chargé avant `app.js`, mais l'appel a lieu au clic, pas au chargement.

- [ ] **Étape 1 : écrire le test**

Dans `tests/coquille.test.js`, près des tests d'envois :

Le test voisin existe déjà — « le canevas prend la couleur du papier visé »
(`coquille.test.js:2035`) — mais il vise le **premier** papier de KDP, le crème : il passe
que la teinte vienne de l'arbre ou de la table, et ne distingue donc rien. Le nouveau vise
le **second**.

```js
/**
 * La teinte suit le papier du livrable, et non le premier de son POD.
 *
 * Le test voisin ne peut pas le dire : il vise le crème, qui est aussi le premier papier
 * de la table. Un canevas qui lirait encore la table plate — laquelle décrit un gabarit,
 * partagé par les deux papiers — replierait sur le crème pour un livre imprimé sur blanc,
 * et personne ne le verrait avant le tirage.
 */
test('le canevas prend la teinte du second papier, pas celle du premier du POD', async () => {
  const a = atelier({
    providers: [KDP],
    livrables: [dest(KDP, 'blanc')],
    sur: {
      envois: {
        gabarit: '',
        liste: [{ dedicataire: 'Léa', main: { mode: 'image' }, place: PLACE_DEFAUT,
          contenu: '', image: 'Léa.jpg', detourage: { papier: 240, encre: 40 } }],
      },
    },
  });
  const { els } = await charge({ invoke: a.invoke });
  await els.get('btNouveau').declenche('click');
  await allerAuxEnvois(els);

  assert.equal(els.get('canevas').style.getPropertyValue('--papier-canevas'), '#ffffff',
    'le canevas prend le crème du premier papier au lieu du blanc du livrable');
});
```

`dest(KDP, 'blanc')` suppose que la fabrique `dest` accepte un papier ; si elle n'en prend
pas, composer le livrable à la main sur son modèle — clé `kdp-6x9-broche-blanc`, gabarit
`kdp-6x9-broche`, papier `blanc`. Ne pas appeler `teintePapier` directement : un test qui
appelle la fonction privée ne prouve pas que l'écran l'utilise.

- [ ] **Étape 2 : lancer et voir échouer**

```bash
node --test tests/coquille.test.js
```

Attendu : échec — le fixture `KDP` porte encore ses papiers dans la table plate, et la
teinte lue est celle du premier (`#f7f0e0`).

- [ ] **Étape 3 : basculer `envois.js` sur l'arbre**

```js
/**
 * La couleur du papier que le livrable visé imprimera.
 *
 * L'arbre et non la table plate : le papier fait partie de l'identité du livrable, jamais
 * du gabarit, et deux papiers d'un même gabarit partagent la ligne de table. Le blanc
 * final ne sert que le cas où l'on n'aurait pas encore le catalogue — mieux vaut un
 * canevas honnêtement blanc qu'un crème inventé.
 */
function teintePapier() {
  return papierCourant()?.teinte ?? '#ffffff';
}
```

- [ ] **Étape 4 : retirer `papiers` de `ProviderVue`**

Dans `commands.rs`, supprimer le champ `papiers: Vec<PapierVue>` de `ProviderVue` et son
remplissage. `PapierVue` **reste** : `pods_liste` la sert dans l'arbre. Le compilateur nomme
les sites à corriger.

- [ ] **Étape 5 : mettre les fixtures d'accord**

Retirer `papiers` des providers plats de `tests/coquille.test.js` (`LULU`, `KDP`,
`COOLLIBRI`), `tests/composition.test.js` et `tests/contrats.test.js`. Les papiers restent
dans les constantes `PODS`, qui décrivent l'arbre. Un fixture qui garde un champ que la vue
ne sert plus décrit un monde qui n'existe pas.

- [ ] **Étape 6 : lancer et voir passer**

```bash
node --test tests/*.test.js
cd src-tauri && cargo test
```

- [ ] **Étape 7 : vérifications et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo run --example temoin
cd .. && touch src-tauri/src/lib.rs
node --test tests/*.test.js
git add src/envois.js src-tauri/src/commands.rs tests/
git commit -m "La table plate cesse de porter des papiers, et le canevas s'en passe"
```

---

### Tâche 5 : Un livrable élagué ne disparaît plus sans un mot

**Fichiers :**
- Modifier : `src-tauri/src/projet.rs` (`normalise` ~l. 416, `lire` ~l. 925)
- Modifier : `src-tauri/src/commands.rs` (`Ouvert`, `ProjetVue`)
- Modifier : `src/livraison.js`, `src/index.html`

`normalise` retire un livrable dont un axe a disparu du catalogue, et repose le livre sur un
autre papier, en silence. Éditer un fichier du poste fait donc disparaître un livrable à la
réouverture, sans une ligne. Le précédent à suivre est dans le même écran : les fichiers de
catalogue refusés au démarrage s'affichent dans `#refusCatalogue` (`livraison.js:21-50`).

- [ ] **Étape 1 : écrire le test Rust**

Dans le module de tests de `projet.rs`, à côté des autres tests de `normalise` :

```rust
    /// Élaguer un livrable se dit. C'est la seule trace qu'un `.ozalid` ouvert sur une
    /// machine dont le catalogue a changé laisse à qui le rouvre : sans elle, un livrable
    /// réglé disparaît entre deux ouvertures et le livre paraît s'être défait tout seul.
    #[test]
    fn un_livrable_elague_est_nomme() {
        let mut l = Livraison {
            livrables: vec![
                Livrable::pour(Fabrication {
                    pod: "bod".into(),
                    format: "135x215".into(),
                    reliure: "broche".into(),
                    papier: "creme-90".into(),
                }),
                Livrable::pour(Fabrication {
                    pod: "pod-parti".into(),
                    format: "a".into(),
                    reliure: "broche".into(),
                    papier: "p".into(),
                }),
            ],
            ..Default::default()
        };
        l.courant = l.livrables[0].cle();

        let elagues = l.normalise();

        assert_eq!(l.livrables.len(), 1, "le livrable orphelin part");
        assert_eq!(elagues.len(), 1, "et il est nommé");
        assert!(
            elagues[0].contains("pod-parti"),
            "le message doit permettre de retrouver ce qui a disparu : {}",
            elagues[0]
        );
    }
```

- [ ] **Étape 2 : lancer et voir échouer**

```bash
cd src-tauri && cargo test un_livrable_elague_est_nomme
```

Attendu : échec de compilation — `normalise` ne rend rien.

- [ ] **Étape 3 : faire rendre `normalise`**

Changer sa signature en `fn normalise(&mut self) -> Vec<String>`, accumuler dans le
`retain_mut` la clé de chaque livrable écarté, et la rendre :

```rust
    /// Remet la liste d'accord avec le catalogue, et **rend ce qu'elle a retiré**.
    ///
    /// Élaguer vaut mieux que refuser d'ouvrir : le reste du projet est intact. Mais
    /// élaguer sans le dire laisse croire que le livre s'est défait tout seul — la liste
    /// rendue remonte jusqu'à l'écran, qui la montre comme il montre les fichiers de
    /// catalogue refusés.
    ///
    /// Le repli de papier n'est pas un élagage : le livrable reste, sous un autre papier.
    fn normalise(&mut self) -> Vec<String> {
        let mut elagues = Vec::new();
```

Dans les deux `return false` du `retain_mut`, pousser avant de sortir :

```rust
                    elagues.push(l.cle());
                    return false;
```

et terminer la fonction par `elagues`.

- [ ] **Étape 4 : porter la liste jusqu'à l'écran**

Dans `projet.rs::lire`, remplacer `meta.livraison.normalise();` par la capture, et poser le
résultat sur `Projet` — un champ public non sérialisé, comme `Ouvert::candidat` vit hors du
projet :

```rust
    /// Les livrables que l'ouverture a retirés faute de catalogue qui les porte encore.
    ///
    /// **Hors du `.ozalid`** : c'est un fait de cette ouverture-ci, sur cette machine-ci,
    /// et le réécrire dans l'archive le ferait resurgir sur une autre où le catalogue est
    /// complet.
    pub elagues: Vec<String>,
```

Dans `commands.rs`, exposer le champ sur `ProjetVue` et le remplir depuis `Ouvert`. Le
compilateur nomme les sites de construction à compléter.

- [ ] **Étape 5 : l'afficher**

Dans `index.html`, à côté de `#refusCatalogue`, une boîte `#livrablesElagues` (masquée par
défaut, `hidden`). Dans `livraison.js`, la remplir sur le même modèle que les refus :

```js
/**
 * Ce que l'ouverture a retiré de la liste. Même traitement que les fichiers de catalogue
 * refusés : on ne peut pas rétablir un livrable dont le catalogue ne porte plus l'axe,
 * mais on peut dire lequel, pour que la disparition se comprenne et se corrige.
 */
function majElagues(vue) {
  const box = $('livrablesElagues');
  box.hidden = (vue.elagues ?? []).length === 0;
  box.textContent = box.hidden
    ? ''
    : `Retiré à l'ouverture, faute de catalogue qui le porte : ${vue.elagues.join(', ')}.`;
}
```

et l'appeler là où la vue du projet est affichée.

- [ ] **Étape 6 : le test front**

Dans `tests/coquille.test.js` :

```js
/**
 * Un livrable élagué à l'ouverture se lit à l'écran. Sans cette ligne, éditer un fichier
 * du poste fait disparaître un livrable à la réouverture et le livre paraît s'être défait
 * tout seul — c'est la dette que le lot 3 a laissée, et que la reliure réglable a élargie.
 */
test("les livrables retirés à l'ouverture sont nommés à l'écran", async () => {
  const { els } = await charge({
    invoke: atelier({ sur: { elagues: ['bod-135x215-broche-papier-parti'] } }).invoke,
  });
  await els.get('btNouveau').declenche('click');

  assert.equal(els.get('livrablesElagues').hidden, false);
  assert.match(els.get('livrablesElagues').textContent, /papier-parti/);
});
```

Le harnais `atelier` doit propager `sur.elagues` dans la vue qu'il rend ; l'ajouter là où il
compose déjà `livraison`.

- [ ] **Étape 7 : lancer les deux suites**

```bash
cd src-tauri && cargo test un_livrable_elague_est_nomme && cargo test
cd .. && node --test tests/coquille.test.js && node --test tests/*.test.js
```

- [ ] **Étape 8 : vérifications et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo run --example temoin
cd .. && node --test tests/*.test.js
git add src-tauri/src/projet.rs src-tauri/src/commands.rs src/livraison.js src/index.html tests/
git commit -m "Un livrable qui disparaît à l'ouverture le dit"
```

---

### Tâche 6 : Le mot « prestataire » quitte le dépôt

**Fichiers :** tout ce que le relevé nomme — commentaires Rust, JSDoc, `README.md`,
`docs/COOKBOOK.md` (titre compris), noms de tests.

Le lot 3 a fait le même travail pour « destinataire » : mécanique mais large — 32
commentaires Rust et 7 passages de README. Celui-ci porte en plus le titre du cookbook et
un chapitre entier.

- [ ] **Étape 1 : relever l'ampleur avant de toucher quoi que ce soit**

```bash
grep -rn "prestataire" --include="*.rs" --include="*.js" --include="*.html" --include="*.md" \
  --include="*.toml" . | grep -v "^./target" | grep -v "^./build" | wc -l
grep -rln "prestataire" --include="*.rs" --include="*.js" --include="*.html" --include="*.md" \
  --include="*.toml" . | grep -v "^./target" | grep -v "^./build"
```

Noter le compte dans le message de commit : c'est ce qui permettra de vérifier que rien
n'est resté.

- [ ] **Étape 2 : choisir le remplaçant selon le contexte**

Le mot ne se remplace pas mécaniquement par un seul terme :

- **« prestataire » désignant l'entreprise** → **« imprimeur »**, le mot du métier, déjà
  employé par `index.html:310` (« Imprimeur à ajouter ») et par `bod.toml`.
- **« prestataire » désignant l'entrée de catalogue** → **« POD »**, qui est le nom du type
  (`catalogue::Pod`) et de la commande (`pods_liste`).
- **« le guide du prestataire »** → **« le guide de l'imprimeur »**.

Ne pas traduire ce qui cite une valeur de données ou un nom de fichier historique.

- [ ] **Étape 3 : appliquer, fichier par fichier**

Ne pas lancer un `sed` global : chaque occurrence se lit avant d'être remplacée, parce que
le choix entre « imprimeur » et « POD » dépend de la phrase. Traiter dans l'ordre :
`src-tauri/src/*.rs`, `src/*.js`, `src/index.html`, `README.md`. **Laisser `docs/COOKBOOK.md`
à la tâche 7**, qui le réécrit largement — le renommer ici ferait deux passes sur les mêmes
lignes.

- [ ] **Étape 4 : vérifier qu'il ne reste rien hors du cookbook**

```bash
grep -rn "prestataire" --include="*.rs" --include="*.js" --include="*.html" --include="*.md" \
  --include="*.toml" . | grep -v "^./target" | grep -v "^./build" | grep -v "COOKBOOK"
```

Attendu : **aucune ligne**.

- [ ] **Étape 5 : vérifications et commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cargo run --example temoin
cd .. && node --test tests/*.test.js
git add -u
git commit -m "Le mot prestataire quitte le code, l'écran et le README"
```

`git add -u` et non `git add -A` : le lot 3 a commité par erreur un fichier non suivi de
`docs/` avec un `git add -A docs/`.

---

### Tâche 7 : Le COOKBOOK parle du monde en fichiers

**Fichiers :**
- Modifier : `docs/COOKBOOK.md` — l. 1-16 (cadre), 85-127 (chapitre BoD), 288-316 (ajouter)

- [ ] **Étape 1 : les quatre pointeurs morts**

`src-tauri/src/providers.rs` n'existe plus. Corriger :

- **l. 8** : « `src-tauri/src/providers.rs` **fait foi** » → « `src-tauri/pods/*.toml` **fait
  foi** — les fichiers embarqués, et ceux que le poste dépose par-dessus dans le répertoire
  de configuration. »
- **l. 68** : « table `providers` » → « `pods/bod.toml` ».
- **l. 76** : « La compléter dans `providers.rs`, depuis le guide. » → « La compléter dans
  `pods/<clé>.toml`, depuis le guide de l'imprimeur. »
- **l. 290** : voir l'étape 3, le paragraphe entier est réécrit.

- [ ] **Étape 2 : le chapitre BoD**

Son tableau « Gabarit, pour mémoire » redit ce que `bod.toml` porte désormais. Le réduire à
ce que le catalogue **ne peut pas** porter :

- La ligne « Format | 13,5 × 21,5 cm » devient « Format | dix, du 12 × 19 au 21 × 29,7 ».
- La ligne « Papier » cite les quatre, et **dit le plafond** : « photo brillant 130 g :
  868 pages au lieu de 900 — l'application le refuse à la génération, en nommant le
  papier ».
- Le piège « Le dos dépend du papier » perd son énumération d'épaisseurs — elles sont en
  table, avec leur source — et garde ce qu'il apprend : changer de papier à la commande sans
  refaire la couverture donne un dos faux.
- Le piège du PDF/X-3 reste tel quel : c'est une réserve sur ce que l'application produit.
- Ajouter ce que le pelliculage change à la composition : **rien**. Trois finitions au
  catalogue, aucune géométrie ; le livrable en garde la trace pour la commande.

- [ ] **Étape 3 : réécrire « Ajouter un prestataire »**

Le chapitre décrit une table Rust à compléter et la fusion de deux tables historiques. Le
remplacer par « Ajouter un imprimeur », qui décrit un fichier :

- **Où** : `src-tauri/pods/<clé>.toml` pour un imprimeur livré avec l'application ; le
  répertoire de configuration du poste pour une surcharge locale — on dépose, on relance.
- **La forme** : renvoyer à la spec § 2 plutôt que de la recopier, et donner le squelette
  minimal — un format, une reliure composable, un papier.
- **Les trois règles d'écriture** : `source` dit d'où vient le chiffre ; `non_outille`
  décrit **notre** état et jamais celui de l'imprimeur ; une valeur d'énumération inconnue
  est refusée, jamais ignorée.
- **Le grisé**, dette du lot 3 : une reliure sans `geometrie` paraît grisée à l'écran, avec
  la phrase de `non_outille` en clair sous elle. C'est cette phrase que l'utilisateur lira —
  elle se rédige pour être lue, pas pour cocher un champ.
- **Ce qui refuse au chargement** : lister les contrôles de `Pod::verifie`, pour qu'un
  fichier refusé se corrige sans lire le Rust.
- **Le repli sans formule de dos** : `dos = { forme = "mesure" }`, le cas de CoolLibri —
  mieux vaut saisir une valeur lue qu'inscrire une formule devinée. C'est déjà écrit, mais
  en syntaxe Rust : le passer en TOML.

- [ ] **Étape 4 : achever le renommage laissé par la tâche 6**

Titre compris. Puis :

```bash
grep -rn "prestataire" docs/COOKBOOK.md
```

Attendu : **aucune ligne**.

- [ ] **Étape 5 : relire le cookbook contre le catalogue**

Pour chacun des six imprimeurs, vérifier qu'aucune valeur du cookbook ne contredit son
`.toml`. Le cookbook cite des sources ; il ne doit plus faire foi sur un chiffre.

- [ ] **Étape 6 : commit**

```bash
git add docs/COOKBOOK.md
git commit -m "Le cookbook parle de fichiers déposés, et cesse de redire la table"
```

---

### Tâche 8 : La spec rejoint ce qui a été fait

**Fichiers :**
- Modifier : `docs/superpowers/specs/2026-08-26-catalogue-et-livrables-design.md` (§ 2, § 10)
- Modifier : `docs/superpowers/plans/2026-08-27-catalogue-lot-4-bod-et-cookbook.md` (ce fichier)

- [ ] **Étape 1 : cocher le lot 4**

Dans le § 10, sur le modèle du lot 3 :

```markdown
**Lot 4 — BoD complété.** ✅ *Fait le 27/08/2026.* Tous ses formats, papiers et reliures,
chacun avec sa `source` relevée dans ses guides. Le COOKBOOK suit.
```

- [ ] **Étape 2 : porter au § 2 le champ que le lot a ajouté**

La spec § 2 décrit le fichier d'un POD et n'y montre pas `pages` sur le papier. L'ajouter à
l'exemple, avec la phrase qui le justifie : la pagination admise vit sur la reliure, **et le
papier peut la resserrer** quand son épaisseur l'impose — BoD plafonne le photo brillant
130 g à 868 pages là où son broché va à 900. Les deux bornes se croisent.

- [ ] **Étape 3 : cocher les cases de ce plan**

Toutes les étapes réellement faites, et **seulement** celles-là. Une case cochée pour une
étape sautée est un mensonge que la prochaine session lira comme un fait.

- [ ] **Étape 4 : commit**

```bash
git add docs/superpowers/
git commit -m "La spec dit ce que le papier peut resserrer, et le lot 4 est coché"
```

---

## À l'œil, avant de clore le lot

Ces vérifications ne se font pas au test : elles demandent l'application lancée. La première
est nouvelle et **ne demande aucun POD d'essai** — c'est le catalogue livré qui l'exerce,
pour la première fois du chantier.

1. **Le contrôle de finition s'allume.** Ouvrir la Livraison, ajouter un livrable BoD : le
   contrôle de finition paraît, avec les trois pelliculages. Chez les cinq autres
   imprimeurs, il reste masqué.
2. **La cascade offre dix formats.** À l'ajout, choisir BoD : la liste des formats en porte
   dix, et propose le 13,5 × 21,5 d'office.
3. **Le papier se règle sur la ligne**, quatre choix, et le dos affiché change avec lui —
   un même livre est plus mince en photo brillant qu'en crème.
4. **La reliure rigide est grisée**, avec sa nouvelle raison en clair sous elle.
5. **Vérification 5 du lot 2, la dernière en suspens** : réécrire une marge dans un `.toml`
   du poste, rouvrir un livre déjà composé chez cet imprimeur, et voir le dos se déclarer
   **périmé**. Le POD d'essai `essai-deux-reliures.toml` est toujours dans le répertoire de
   configuration ; c'est lui qu'il faut modifier.
6. **Un livrable élagué se dit** (tâche 5) : retirer de ce même fichier un axe qu'un
   livrable du livre utilise, rouvrir, et lire la ligne qui nomme ce qui a disparu.

## Ce que ce lot ne fait pas

- **La couverture rigide reste non composable.** Le relevé donne pourtant sa géométrie —
  rempli 17 mm, rabat 8 mm — mais `planche` ne sait pas la composer, et la spec range ce
  chantier hors périmètre. La tâche 2 corrige seulement la raison écrite, qui prétendait à
  tort que le relevé manquait.
- **Les trois papiers réservés aux éditeurs et à l'interface FTP** restent hors table : hors
  du parcours myBoD, ils ne se commandent pas.
- **Les cinq imprimeurs tier B et C** du comparatif restent hors périmètre.
- **Les neuf autres imprimeurs ne sont pas complétés.** Le lot ne traite que BoD ; ce que le
  relevé a appris sur la méthode vaut pour les autres, et la reconnaissance § 5 la décrit.
