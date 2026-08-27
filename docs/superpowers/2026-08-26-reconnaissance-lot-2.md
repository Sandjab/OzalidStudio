# Reconnaissance du lot 2 — le livrable à cinq axes

Fait le 26/08/2026 sur `catalogue-en-fichiers`. Toutes les expériences tournent sur une
copie (`git archive HEAD | tar -x`) dans le scratchpad ; **le dépôt n'a pas été touché**.

> ⚠ **La branche a avancé pendant la reconnaissance.** Les expériences portent sur
> `11c6f46` (« Un fichier de catalogue refusé se nomme à la Livraison »). Le HEAD est
> maintenant `13bdd13`, après trois commits — `dadec0f` (le corps et l'interligne quittent
> le catalogue), `80782c3` (la ligne de refus), `13bdd13` (les quatorze prestataires
> ancrés). J'ai revérifié toutes mes références contre le nouveau HEAD ; **`commands.rs`,
> `projet.rs` et `package.rs` sont inchangés au caractère près**, donc toutes les lignes
> citées pour eux tiennent. Trois décalages et deux faits neufs sont notés au § 8 en fin
> de rapport. Relève-les avant d'écrire le plan : l'un d'eux change une conclusion.

**Comment les expériences ont été montées** — elles vivaient dans un scratchpad de session
qui ne survit pas ; le dispositif est décrit ici pour être refait, et le code des esquisses
est reproduit en clair dans les verdicts plutôt que cité par un chemin mort.

- L'arbre : `git archive HEAD | tar -x -C <ailleurs>`, **plus `src-tauri/binaries/` et
  `src-tauri/fonts/` recopiés à la main**. Ces deux répertoires ne sont pas trackés, et
  sans eux `cargo check` échoue au `build.rs` de `tauri-build` :
  `resource path 'binaries/typst-aarch64-apple-darwin' doesn't exist`, puis
  `glob pattern fonts/* path not found`. C'est le premier obstacle et il coûte deux
  compilations complètes si on ne le sait pas.
- Le `target` : un `CARGO_TARGET_DIR` à part, **jamais celui du dépôt** — le `target/` du
  dépôt fait 6,5 Go et son verrou aurait bloqué la tâche qui travaillait en parallèle.
- Un crate `serde` + `toml 0.8` + `serde_json` isolé pour les essais de sérialisation :
  ils ne demandent pas Tauri, et compilent en secondes au lieu de minutes.

---

# Première partie — la carte

## 1. `Destinataire`, `Livraison`, `Mesure`

### Les trois types

| | fichier:ligne |
|---|---|
| `struct Destinataire { provider, papier, dos_mm, fond_perdu_mm, compose }` | `src-tauri/src/projet.rs:238` |
| `struct Mesure { pages, gouttiere, blanche, dos, polices_introuvables }` | `src-tauri/src/projet.rs:264` |
| `impl Destinataire::pour(&Provider)` | `src-tauri/src/projet.rs:288` |
| `struct Livraison { destinataires: Vec<Destinataire>, courant: String, deja_compose: bool }` | `src-tauri/src/projet.rs:305` |
| `impl Default for Livraison` | `src-tauri/src/projet.rs:323` |
| `Livraison::courant() -> Option<&Destinataire>` | `src-tauri/src/projet.rs:336` |
| `Livraison::oublier_mesures()` | `src-tauri/src/projet.rs:351` |
| `Livraison::retenir_mesure(&str, Mesure)` | `src-tauri/src/projet.rs:361` |
| `Livraison::normalise()` — **privée** | `src-tauri/src/projet.rs:379` |

### Les invariants documentés, mot pour mot

- **« Une mesure présente vaut toujours »** (`projet.rs:246-251`) : rien n'est estampillé,
  rien n'est à comparer avant usage, et ce qui pourrait la périmer l'efface à la source.
- **Le pointeur n'est jamais vide** (`projet.rs:320-322`) : un livre naît avec un
  destinataire, parce qu'apercevoir une couverture réclame un format sans réclamer de
  composition.
- **`courant` désigne toujours l'un des destinataires** (`projet.rs:307-308`).
- **`deja_compose` est de l'histoire, jamais un état courant** (`projet.rs:311-317`) :
  posé à la première composition, jamais repris. Il distingue « jamais demandé » de
  « périmé ».
- **`Mesure` n'est plus `Copy`** depuis qu'elle porte `polices_introuvables: Vec<String>`
  (`projet.rs:258-262`). Elle se lit par `.clone()` derrière une référence — c'est
  exactement le genre de piège qui a coûté un aller-retour au lot précédent.

### Qui appelle les trois fonctions

`Livraison::default` — `projet.rs:553` (`Projet::nouveau`), et par `#[serde(default)]` sur
`Metadonnees::livraison` (`projet.rs:422`) : **tout `.ozalid` sans section `[livraison]`
passe par là**.

`normalise` — un seul appel hors tests : `projet.rs:774`, dans `Projet::lire`, juste après
`migre`. C'est le point d'entrée unique de tout `.ozalid` ouvert.

`oublier_mesures` — trois appels, tous dans `projet.rs`, tous délibérément placés dans le
mutateur et non chez l'appelant (`projet.rs:569-570` le dit) :
`modifier_livre` (573), `modifier_interieur` (579), `remplacer_texte` (586).

`retenir_mesure` — un seul appel hors tests : `commands.rs:647`, dans `composer`.

## 2. Tout ce qui lit `destinataire.provider` ou `livraison.courant`

### Rust — liste complète

`.provider` :

- `projet.rs:239` (déclaration), `290` (`pour`), `339` (`courant()`), `365`
  (`retenir_mesure`), `382` et `392` et `397` (`normalise`)
- `commands.rs:488-489` (`vise`), `504` (`destinataire_ajouter`), `529` et `538`
  (`destinataire_retirer`), `549-550` et `560` (`destinataire_regler`), `583`
  (`destinataire_viser`), `1286-1292` et `1312` et `1321` (`packager`), `2015-2016`
  (`vue`, le lien du PDF)
- `package.rs:27` (champ `Package.provider`), `138` (rempli depuis `pr.cle`)
- `commands.rs:1257` (champ `Resultat.provider`)
- `examples/ebook.rs:34`

`.courant` :

- `projet.rs:308` (déclaration), `328`, `339`, `396-397`
- `commands.rs:486` (`vise`), `537-538` (`destinataire_retirer`), `588`
  (`destinataire_viser`), `2012` (`vue`)
- `examples/ebook.rs:32`

Tests Rust : `projet.rs:1357, 1359, 1371, 1378, 1383-1387, 1407, 1411, 1433, 1437, 1475-1481,
1496, 1502, 1530, 1545, 1552, 1558-1568, 1583-1588` ; `commands.rs:2149`.

### JavaScript — liste complète

| fichier:ligne | ce qui est lu |
|---|---|
| `src/app.js:229` | `p.livraison.deja_compose && !destinataireCourant()?.compose` |
| `src/app.js:359, 368-371` | le sélecteur du pied, une `Option` par destinataire |
| `src/app.js:396` | `destinataireCourant()?.compose` — le pied |
| `src/app.js:485-486` | `providerCourant()` : joint `providers` sur `livraison.courant` |
| `src/app.js:490-491` | `destinataireCourant()` : `find(d => d.provider === livraison.courant)` |
| `src/app.js:496` | `libelleProvider(cle)` |
| `src/app.js:531` | `destinataireCourant()?.compose?.polices_introuvables` |
| `src/app.js:600, 622` | `destinataireCourant()?.compose` — **la veille et la recomposition** |
| `src/app.js:1259-1260, 1265-1266` | `destinataire_viser` / `destinataire_ajouter` |
| `src/couverture.js:687` | `destinataireCourant()?.compose?.dos` |
| `src/couverture.js:833-834` | le format de l'aperçu, joint sur `d?.provider` |
| `src/envois.js:367-368` | la teinte du papier, jointe sur `x.provider` et `l.courant` |
| `src/livraison.js:64-111` | la liste entière : `d.provider` sert d'**identifiant de DOM** (`dest-papier-${d.provider}`, `dest-retirer-${d.provider}`, `dest-${quoi}-${d.provider}`) |
| `src/livraison.js:131-139, 144-166` | `reglerDestinataire(cle)` → `destinataire_regler` |
| `src/livraison.js:239` | le compte de destinataires |

**Point à retenir :** `d.provider` n'est pas seulement une donnée, c'est le **suffixe des
`id` du DOM** dans `livraison.js`. Une identité à quatre axes doit produire une chaîne
unique et valide en `id` HTML.

## 3. Les commandes, signatures exactes

```rust
// commands.rs:479 — nommée `vise`, pas `couple`
fn vise(o: &Ouvert)
    -> Result<(&'static Provider, &'static catalogue::Papier, &Destinataire), String>

// commands.rs:1904
fn papier(pr: &'static Provider, cle: Option<&str>)
    -> Result<&'static catalogue::Papier, String>

// commands.rs:495
pub fn destinataire_ajouter(provider_cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String>
// commands.rs:514
pub fn destinataire_retirer(provider_cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String>
// commands.rs:545
pub fn destinataire_regler(destinataire: Destinataire, atelier: State<Atelier>) -> Result<ProjetVue, String>
// commands.rs:576
pub fn destinataire_viser(provider_cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String>
```

> **Renseignement :** la spec (§ 3) et ton brief parlent de `couple`. Cette fonction
> n'existe pas ; c'est `vise`. Et les deux `&'static` ne sont pas à `commands.rs:467` et
> `1890` comme la spec l'annonce mais à **479** et **1904** — la spec a été écrite avant
> les derniers commits du lot 1.

**Appelants de `vise` — neuf, tous dans `commands.rs`** : 598 (`composer`), 970, 1104,
1214, 1341 (`ebook_generer`), 1681, 1771, 1819, 1875. Sept d'entre eux jettent le
destinataire (`let (pr, _, _)`), deux le gardent (`970`, `1341`, `1681`, `1819`, `1875`
gardent `d` pour les relevés).

**Appelants de `papier`** : `vise` (490), `destinataire_regler` (551), `packager` (1296).

## 4. `package.rs` : les noms

```rust
// package.rs:54
fn nom(pr: &Provider, quoi: &str, ext: &str) -> String {
    format!("{quoi}-{}.{ext}", pr.cle)
}
```

Cinq fichiers en sortent, tous dans le même répertoire :
`interieur-{cle}.typ`, `interieur-{cle}.pdf`, `couverture-{cle}.typ`,
`couverture-{cle}.pdf`, `couverture-{cle}.png`.

Le **répertoire**, lui, est fabriqué chez l'appelant, jamais dans `package.rs` :

```rust
// commands.rs:1940
fn sorties_dossier(o: &Ouvert, provider: &str) -> Result<PathBuf, String> {
    Ok(sorties_racine(o)?.join(provider))
}
// commands.rs:1949
fn interieur_pdf(dossier: &Path, provider: &str) -> PathBuf {
    dossier.join(format!("interieur-{provider}.pdf"))
}
```

Ce qui en dépend ailleurs :

- `commands.rs:606` (`composer` écrit) et `commands.rs:2015-2016` (`vue` reconstruit le
  même chemin pour en faire un lien). **Deux endroits fabriquent le même chemin depuis la
  même clé** — le commentaire de `interieur_pdf` (1945-1947) dit explicitement qu'il est
  nommé là pour qu'il n'y ait pas deux `format!` qui divergent.
- `commands.rs:1297` (`packager`), `commands.rs:1877` (les envois : `racine.join("envois")`).
- `examples/packager.rs:41` (`racine.join(&pr.cle)`), `examples/composer.rs:45, 72`.
- Test `package.rs:497` : `les_sorties_portent_la_cle_du_prestataire`.

## 5. Le `.ozalid`

### La forme sérialisée réelle

`build/travail/candide.ozalid` (non versionné, ouvert en lecture seule) :

```toml
[ozalid]
version = 4
…
[livraison]
courant = "lulu"
deja_compose = false

[[livraison.destinataires]]
provider = "lulu"
papier = "standard"
```

C'est le **cas de migration réel** : un seul destinataire, `lulu` / `standard`, aucun
relevé, aucune mesure, `deja_compose = false`.

### Le mécanisme de tolérance existe déjà, et il est exactement celui qu'il faut

`VERSION: u32 = 4` (`projet.rs:51`). Trois dispositifs se superposent :

1. **`fn migre(toml::Value) -> Result<Metadonnees, String>`** (`projet.rs:458`) : opère sur
   le `toml::Value`, **avant** la désérialisation typée, précisément parce que « en v3,
   `Couverture` ne porte plus ces champs, il n'y a donc plus de structure Rust capable de
   les lire ». Elle porte déjà deux migrations (v2→v3 les textes de la maquette, v3→v4 la
   main des envois). Elle est idempotente : un projet déjà en v4 traverse sans bouger, et
   les `entry().or_insert()` protègent une migration rejouée (`projet.rs:508`).
2. **Le refus du futur avant la migration** (`projet.rs:766-772`) : `version > VERSION`
   est refusé, et le commentaire dit pourquoi l'ordre compte.
3. **`normalise()`** (`projet.rs:379`) : élague ce que le catalogue ne porte plus, plutôt
   que de refuser d'ouvrir. C'est la seconde ligne de défense, celle qui rattrape ce que
   la migration n'a pas su convertir.

**Rien n'est à inventer** : le lot 2 monte `VERSION` à 5 et ajoute une branche dans
`migre`. La forme est déjà écrite deux fois dans le fichier.

## 6. Les tests qui casseront

### Rust — 464 tests dans la lib, 1 test d'intégration

`cargo test --lib` : **464 passés, 3 ignorés** (mesuré sur la copie).
Plus `src-tauri/tests/catalogue_initialise.rs` — 1 test d'intégration, qui vérifie que le
démarrage charge les fichiers du poste et refuse un second chargement. **Il ne peut pas
vivre dans la lib**, parce que `initialiser` pose un `OnceLock` une fois pour le
processus ; si le lot 2 change `initialiser`, c'est ce fichier-là qui bouge.

**76 tests** citent une clé plate littérale (`"lulu"`, `"bod"`, `"kdp-*"`,
`"coollibri-*"`) ou `catalogue::provider(…)`. Mais **la plupart passent par un helper** :

| module | helper | ligne |
|---|---|---|
| `planche.rs` | `fn gabarit(cle, pages)` — `provider(cle).unwrap()` puis `Gabarit::pour` | `610`, `630-633` |
| `interieur.rs` | `use crate::catalogue::provider` + appels directs | `603` |
| `package.rs` | `use crate::catalogue::provider` | `403` |
| `ebook.rs` | `use crate::catalogue::provider` | `242` |
| `maquettes.rs` | `for pr in crate::catalogue::providers()` | `923` |

Tant que `catalogue::provider(cle) -> Option<&Provider>` survit, **ces 76 tests ne
bougent pas**. Voir le verdict 1 ci-dessous : c'est jouable.

Ceux qui cassent **quoi qu'il arrive**, parce qu'ils portent la forme du destinataire :

| test | ligne | pourquoi |
|---|---|---|
| `un_projet_sans_section_livraison_prend_le_premier_gabarit` | `projet.rs:1345` | compare `livraison.courant` et `destinataires[0].provider` à `providers()[0].cle` |
| `la_liste_des_destinataires_survit_a_l_aller_retour` | `projet.rs:1365` | construit `Destinataire { provider: "coollibri-148x210", … }` |
| `la_mesure_d_un_destinataire_survit_a_l_aller_retour` | `projet.rs:1403` | `retenir_mesure(&livraison.courant, …)` puis relit `courant().compose` |
| `une_mesure_sans_le_champ_se_relit_vide` | `projet.rs:1449` | TOML littéral d'une `Mesure` |
| `une_mesure_ne_renseigne_que_son_destinataire` | `projet.rs:1472` | **cet invariant change de sens** : la mesure ne sera plus sur le destinataire |
| `ce_qui_pagine_efface_toutes_les_mesures` | `projet.rs:1493` | |
| `perimer_une_mesure_n_efface_pas_l_histoire_du_livre` | `projet.rs:1524` | |
| `une_livraison_incoherente_est_elaguee_plutot_que_refusee` | `projet.rs:1541` | `provider: "prestataire-disparu"` |
| `une_livraison_videe_reprend_le_premier_gabarit` | `projet.rs:1580` | |
| `le_repli_de_police_survit_a_l_aller_retour` | `projet.rs:1425` | |
| `le_destinataire_de_l_interface_se_lit` | `commands.rs:2141` | JSON littéral `{"provider": "coollibri-148x210", …}` — il **existe pour prouver le snake_case de Tauri**, il devra être réécrit, pas supprimé |
| `un_releve_absent_reste_absent` | `commands.rs:2159` | JSON littéral |
| `les_sorties_portent_la_cle_du_prestataire` | `package.rs:497` | le nom des cinq fichiers |
| `un_projet_complet_survit_a_l_aller_retour` | `projet.rs:1284` | |
| `un_projet_v3_traverse_la_migration_sans_bouger` | `projet.rs:1041` | il faudra son jumeau v4→v5 |

Soit **une quinzaine à réécrire**, pas 76 — si et seulement si `Provider` survit comme
type de calcul.

### JavaScript — 247 tests, 236 dans les 9 fichiers exposés

| fichier | tests | lignes touchées |
|---|---|---|
| `tests/coquille.test.js` | 76 | 42 |
| `tests/couverture.test.js` | 49 | 12 |
| `tests/packages.test.js` | 36 | 73 |
| `tests/composition.test.js` | 27 | 13 |
| `tests/cycle_de_vie.test.js` | 21 | 3 |
| `tests/contrats.test.js` | 14 | 4 |
| `tests/epreuve.test.js` | 5 | 7 |
| `tests/ebook.test.js` | 4 | 7 |
| `tests/dom_shim.test.js` | 4 | 3 |
| `tests/placement.test.js` | 9 | **0** — le seul épargné |

Deux fichiers tiennent la livraison « pour de vrai » et sont les plus coûteux :

- `tests/coquille.test.js:65-190` — le faux Rust : il tient `livraison` comme un état,
  implémente `destinataire_viser` (161), `destinataire_regler` (164), et efface `compose`
  comme le vrai le fait ;
- `tests/packages.test.js:107-190` — même chose, plus `destinataire_ajouter` (169) et
  `destinataire_retirer` (177).

Les sept autres servent un projet figé (`const PROJET = { livraison: { destinataires: [{
provider: 'lulu', papier: 'standard', … }], courant: 'lulu' } }`).

> **⚠ Verdict d'expérience — le renommage naïf ne fait pas échouer les tests, il les fait
> boucler.** J'ai renommé mécaniquement `provider:` → `pod:` et `.provider` → `.pod` dans
> les 9 fichiers de la copie, puis lancé `node --test tests/*.test.js` : **la suite ne
> termine pas** (tuée après 5 min, puis fichier par fichier après 10 min).
>
> La cause est `app.js:596-601` (`veiller`) et `app.js:621` (`recomposer`) :
>
> ```js
> if (!(consenti || projet?.livraison.deja_compose) || destinataireCourant()?.compose) return;
> ```
>
> Quand `destinataireCourant()` rend `undefined` — parce que le `find` sur `d.provider`
> ne trouve plus rien —, `?.compose` est `undefined`, la garde ne retient plus, la veille
> s'arme, `composer()` s'exécute, `afficherProjet` rappelle `veiller()`, et ainsi de
> suite. **C'est vrai dans l'application aussi**, pas seulement dans les tests : si
> `livraison.courant` cesse de désigner un livrable de la liste, Ozalid recompose en
> boucle sans qu'aucune erreur ne paraisse.

## 7. Ce que dit `build/travail/candide.ozalid`

Voir § 5 ci-dessus. À noter :

- `deja_compose = false` et **aucune** section `[livraison.destinataires.compose]` : le cas
  réel n'exerce donc **pas** le déplacement de la mesure. Il faudra un `.ozalid` fabriqué
  pour ça (mon essai en construit un, § verdict 3).
- `[manuscrit] source` est un **chemin absolu** vers `build/in/texts/candide.md`.
- `lulu` est la première clé de `FOURNIS` (`catalogue.rs:535`) et donc
  `providers()[0]` : c'est ce que `Livraison::default()` pose. Le format `108x175` de
  Lulu n'a **qu'une tranche de gouttière publiée, 151–400 pages** : un livre neuf hors de
  cette tranche refuse déjà de composer aujourd'hui. Ce n'est pas une régression du lot 2,
  mais si le plan touche à `Livraison::default`, c'est le piège où tomber.

---

# Deuxième partie — ce qui ne compilera pas

Tout ce qui suit a été **compilé et exécuté**. Les messages sont ceux du compilateur.

## Verdict 1 — l'identité à quatre axes, et les deux `&'static`

### 1a. Les quatre `&'static` s'extraient sans peine — ✅ compile

L'esquisse a été posée en `src-tauri/src/esquisse.rs`, branchée dans `lib.rs`.
`cargo check --lib` : **Finished, aucune erreur.** La voici **en entier** — c'est elle qui
porte le verdict, et le fichier d'origine ne survit pas à la session :

```rust
use std::sync::OnceLock;
use crate::catalogue::{Format, Papier, Pod, Reliure};

/* ---------- 1. le catalogue à cinq axes derrière un OnceLock ---------- */

static PODS: OnceLock<Vec<Pod>> = OnceLock::new();

pub fn pods() -> &'static [Pod] {
    PODS.get_or_init(|| crate::catalogue::fournis().expect("catalogue fourni illisible"))
}

/* ---------- 2. l'identité, telle que le .ozalid la porte ---------- */

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Fabrication {
    pub pod: String,
    pub format: String,
    pub reliure: String,
    pub papier: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Livrable {
    #[serde(flatten)]
    pub fabrication: Fabrication,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dos_mm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fond_perdu_mm: Option<f64>,
}

/* ---------- 3. le livrable résolu contre le catalogue ---------- */

/// Quatre références dans une table qui vit aussi longtemps que le processus.
/// `Copy` : rien ici n'est possédé.
#[derive(Debug, Clone, Copy)]
pub struct Resolu {
    pub pod: &'static Pod,
    pub format: &'static Format,
    pub reliure: &'static Reliure,
    pub papier: &'static Papier,
}

impl Resolu {
    pub fn fond_perdu(&self) -> Option<f64> {
        self.format.fond_perdu.or(self.pod.fond_perdu)
    }

    pub fn libelle(&self) -> String {
        format!("{} — {}", self.pod.nom, self.format.nom)
    }

    /// Le nom du répertoire de package. Voir le verdict 4 : le `replace` est arbitraire.
    pub fn dossier(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.pod.cle, self.format.cle, self.reliure.cle,
            self.papier.cle.replace('-', "")
        )
    }
}

pub fn resout(f: &Fabrication) -> Result<Resolu, String> {
    let pod = pods().iter().find(|p| p.cle == f.pod)
        .ok_or_else(|| format!("POD inconnu : {}", f.pod))?;
    let format = pod.formats.iter().find(|x| x.cle == f.format)
        .ok_or_else(|| format!("{} ne fait pas le format {}", pod.nom, f.format))?;
    let reliure = pod.reliures.iter().find(|x| x.cle == f.reliure)
        .ok_or_else(|| format!("{} ne fait pas la reliure {}", pod.nom, f.reliure))?;
    // Le refus par le Rust d'une reliure non outillée, que la spec § 9 réclame.
    if reliure.geometrie.is_none() {
        return Err(match &reliure.non_outille {
            Some(raison) => format!("{} : {raison}", reliure.nom),
            None => format!("{} n'est pas composable.", reliure.nom),
        });
    }
    let papier = pod.papiers.iter().find(|x| x.cle == f.papier)
        .ok_or_else(|| format!("papier inconnu chez {} : {}", pod.cle, f.papier))?;
    Ok(Resolu { pod, format, reliure, papier })
}

/* ---------- 4. ce que `vise` devient : quatre 'static + un emprunt court ---------- */

pub fn vise(l: &crate::projet::Livraison) -> Result<(Resolu, &Livrable), String> {
    let _ = l;
    todo!("le lot 2 l'écrira")
}

/* ---------- 5. ce que `converge` réclame, porté sur le format ---------- */

impl crate::catalogue::Format {
    pub fn gouttiere_de(&self, pages: u32) -> Result<f64, String> {
        self.gouttieres.iter()
            .find(|t| t.de <= pages && pages <= t.a)
            .map(|t| t.mm)
            .ok_or_else(|| format!("{pages} pages : tranche absente du format {}", self.cle))
    }
}
```

Trois faits établis :

- `(&'static Pod).formats.iter().find(…)` rend bien un `&'static Format` — la durée de vie
  se propage par les champs, aucun `unsafe`, aucun `Box::leak`.
- `Resolu` est `Copy` : il se passe par valeur partout, contrairement à `Provider` qui
  porte des `String` et un `Vec<Papier>`.
- Le refus par le Rust d'une reliure non outillée (spec § 9) tient en trois lignes dans
  `resout` : `if reliure.geometrie.is_none() { return Err(reliure.non_outille…) }`.

### 1b. Ce qui résiste : **les consommateurs, pas les durées de vie**

`Resolu` ne peut pas remplacer `&Provider` sans réécrire **20 signatures dans 6 modules** :

- `interieur.rs:113` (`converge`), `182` (`assemble`), `428` (`source`), `447`
  (`source_ebook`) — lit `pr.gouttieres[0].2`, `pr.gouttiere(pages)`, `pr.format`,
  `pr.marge_haut`, `pr.marge_bas`, `pr.exterieur`, **et `pr.cle`**
- `planche.rs:44` (`Gabarit::pour`) — lit `pr.format`, `pr.fond_perdu`, `pr.libelle`
- `package.rs:65` (`assembler`), `262` (`assembler_envois`), `54` (`nom`) — lit `pr.cle`,
  `pr.libelle`, `pr.pages_min`, `pr.pages_max`
- `ebook.rs:54` (`generer`) — lit `pr.format`
- `commands.rs:77` (`From<&Provider> for ProviderVue`)
- `projet.rs:288` (`Destinataire::pour`)

Deux pièges dans cette liste :

1. **`pr.format` est un tuple `(f64, f64)`**, pas un `Dimensions`. Il est déstructuré
   (`interieur.rs:188` : `let (fw, fh) = pr.format;`) et passé tel quel à
   `couverture::page_une` (`ebook.rs:101, 113`) et à `Gabarit.format` (`planche.rs:63`,
   lu ensuite en `.0` / `.1` à `planche.rs:70, 74, 93, 411, 924`). Le remplacer par
   `Dimensions { largeur, hauteur }` casse **tous ces sites**, silencieusement au premier
   abord et bruyamment à la compilation. C'est la faute « comparaison à un tuple sur un
   type qui n'en était plus un » qui t'a coûté un aller-retour.
2. **`pr.cle` entre dans la source Typst générée** — `interieur.rs:238`, dans le
   commentaire d'en-tête : `// Intérieur — {titre} ({cle})`. Le test
   `la_source_porte_le_gabarit_du_prestataire_et_le_marqueur` (`interieur.rs:778`) le
   vérifie. Changer la clé change la source composée.

### 1c. L'alternative qui compile en entier — ✅ 464 tests passent

**Expérience faite en vrai sur la copie** : garder `Provider` comme *type de calcul*, mais
lui retirer son `&'static`. `vise` et `papier` rendent des valeurs possédées.

```rust
// avant
fn vise(o: &Ouvert)
    -> Result<(&'static Provider, &'static catalogue::Papier, &Destinataire), String>
fn papier(pr: &'static Provider, cle: Option<&str>)
    -> Result<&'static catalogue::Papier, String>

// après
fn vise(o: &Ouvert) -> Result<(Provider, catalogue::Papier, &Destinataire), String>
fn papier(pr: &Provider, cle: Option<&str>) -> Result<catalogue::Papier, String>
```

Premier `cargo check --lib` : **8 erreurs, toutes de la même famille**, aucune erreur
d'emprunt :

```
src/commands.rs:622:44: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:619:33: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:633:40: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:1301:17: error[E0308]: mismatched types: expected `&Papier`, found `Papier`
src/commands.rs:1343:31: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:1696:45: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:1846:13: error[E0308]: mismatched types: expected `&Provider`, found `Provider`
src/commands.rs:1879:19: error[E0308]: arguments to this function are incorrect
```

**Le correctif est neuf `&` et deux `.clone()`.** Après quoi :

```
cargo check --all-targets   →  Finished, 0 erreur (tests et 8 exemples compris)
cargo test --lib            →  464 passed; 0 failed; 3 ignored
cargo run --example temoin  →  BoD — 13,5 × 21,5 cm — 98 pages, gouttière 20.0 mm,
                               dos 7.21 mm, planche 287.21 × 225.00 mm, blanche de parité
```

C'est-à-dire **la valeur du témoin, inchangée**.

> **Ce que ça veut dire pour ton plan.** Le `&'static` que la spec § 3 présente comme la
> contrainte structurante n'en est pas une : il tombe pour neuf esperluettes. Un `Provider`
> construit à la volée depuis (POD, format, reliure, papier) laisse `interieur`, `planche`,
> `package`, `ebook` **intacts**, et les 76 tests qui passent par `catalogue::provider`
> avec eux. Le lot 2 se réduit alors à trois choses : l'identité que le projet porte, la
> mesure déplacée, et le nom des sorties.
>
> Le prix est nommable : deux `clone()` d'un `Provider` (quelques `String` et un
> `Vec<Papier>`) par commande. C'est une composition Typst de plusieurs secondes qui suit ;
> le coût est nul en pratique.

Deux réserves à porter au plan si tu prends cette voie :

- `providers()` / `provider(cle)` doivent alors se peupler autrement. La vue plate ne peut
  plus être un `Vec<Provider>` figé, puisqu'un couple POD × format n'a plus de papier
  unique. Le plus simple : `PODS: OnceLock<Vec<Pod>>` (verdict 1a) + un
  `fn assemble(pod, format, reliure, papier) -> Provider` qui fabrique. Les tests qui
  appellent `provider("bod")` gardent un helper `#[cfg(test)]` du même nom.
- `ProviderVue` (`commands.rs:55-95`) sert la liste à l'écran. Le lot 3 la remplace par la
  cascade ; le lot 2 peut la laisser telle quelle.

## Verdict 2 — la forme sérialisée de la `Mesure` déplacée

Essais menés dans le crate `serde` + `toml 0.8` isolé, chaque forme sérialisée puis
relue.

| forme | verdict |
|---|---|
| `BTreeMap<String, Mesure>`, clé `"bod/135x215/broche"` | ✅ sérialise, se relit — mais **guillemets obligatoires** dans le TOML |
| `BTreeMap<String, Mesure>`, clé `"bod-135x215-broche"` | ✅ **et sans guillemets** : `bod-135x215-broche = 98` |
| `BTreeMap<String, Mesure>`, clé `"bod.135x215.broche"` | ✅ mais guillemets obligatoires (le point est un séparateur TOML) |
| `Vec<{pod, format, reliure, #[serde(flatten)] mesure}>` | ✅ sérialise, se relit, `dos = 7.21` reste un `f64` exact |
| **`BTreeMap<(String,String,String), Mesure>`** | ❌ **`map key was not a string`** — refus à la sérialisation |

Ce que ça donne à l'œil, dans une `Metadonnees` complète :

```toml
[livraison]
courant = 0
deja_compose = true

[[livraison.livrable]]
pod = "bod"
format = "135x215"
reliure = "broche"
papier = "creme-90"
finition = "mat"

[[livraison.livrable]]
pod = "bod"
format = "135x215"
reliure = "broche"
papier = "blanc-90"

[livraison.mesures."bod/135x215/broche"]
pages = 98
gouttiere = 20.0
blanche = false
dos = 7.21
```

**Recommandation :** la map à clé composée, avec **des tirets et non des barres obliques**
— `[livraison.mesures.bod-135x215-broche]` s'écrit sans guillemets et se lit d'un coup
d'œil. La liste `[[livraison.mesure]]` est lisible aussi mais elle laisse la porte à deux
entrées de même clé, que la map interdit par construction.

Trois pièges vérifiés :

1. **La clé composée ne doit jamais être re-découpée.** Avec des tirets, `tbe-110x170`
   (une clé héritée d'aujourd'hui !) montre que le séparateur apparaît déjà dans les
   valeurs. Fabriquer la clé, la comparer — jamais la parser.
2. **L'ordre de déclaration des champs n'est *pas* un piège** : `toml 0.8`
   `to_string_pretty` réordonne de lui-même (essai D : une `BTreeMap` déclarée avant les
   scalaires ressort quand même après eux). Le piège classique « values must be emitted
   before tables » ne se déclenche pas ici. Vérifié aussi avec une table `courant` posée
   après un `[[livrable]]` : ✅ sérialise et se relit.
3. **`#[serde(flatten)]` marche en TOML *et* en JSON** (le chemin Tauri), le `f64` traverse
   intact. Mais il **désactive le refus des champs inconnus** : `{"pod":…,"inconnu":1}` est
   accepté sans un mot. Si le plan met `deny_unknown_fields` sur `Livrable`, `flatten` le
   rendra inopérant.

**La forme de `courant`**, les trois essayées : les trois se sérialisent
et se relisent — une clé composée (`courant = "bod-135x215-broche-creme-90"`), un index
(`courant = 0`), une table à quatre axes (`[courant]`). L'index est le plus fragile : un
livrable retiré décale le pointeur en silence, et `normalise` doit le rattraper. Vu le
risque de boucle infinie décrit au § 6, **`courant` doit être une clé, pas un index**, et
`normalise` doit garantir qu'elle désigne toujours un livrable de la liste.

## Verdict 3 — la migration : ✅ compile et tourne sur le Candide réel

Essai lancé sur le `projet.toml` réellement extrait de `build/travail/candide.ozalid`, puis
sur un ancien fabriqué à trois destinataires.

Le code tient en une fonction sur le `toml::Value`, dans l'esprit exact de `migre`
(`projet.rs:458`) :

```rust
fn migre_livraison(v: &mut toml::Value) {
    let Some(l) = v.get_mut("livraison").and_then(toml::Value::as_table_mut) else { return };
    let anciens = l.remove("destinataires");
    let ancien_courant = l.get("courant").and_then(toml::Value::as_str).map(str::to_owned);
    let Some(anciens) = anciens.as_ref().and_then(toml::Value::as_array) else { return };
    for d in anciens {
        let cle = …;                          // `provider`
        let Some(&(pod, format, reliure)) = table.get(cle) else { continue };  // élagage
        …                                     // pod / format / reliure + papier, dos_mm, fond_perdu_mm
        if let Some(m) = t.get("compose") {   // la mesure quitte le destinataire
            mesures.entry(format!("{pod}/{format}/{reliure}")).or_insert_with(|| m.clone());
        }
    }
    …
}
```

**Sortie réelle sur le Candide** :

```toml
[ozalid]
version = 5
…
[livraison]
courant = 0
deja_compose = false

[[livraison.livrable]]
pod = "lulu"
format = "108x175"
reliure = "broche"
papier = "standard"
```

**Sur un ancien à trois destinataires** (bod avec mesure, kdp-55x85 avec relevés, plus un
prestataire disparu) : les deux connus sont convertis, le disparu est élagué, `courant`
retombe sur le bon index, la mesure du bod atterrit sous `bod/135x215/broche`, les relevés
du kdp restent sur son livrable. **Rejouée sur son propre résultat, la migration ne bouge
rien** (2 livrables, 1 mesure, même `courant`).

### La table des quatorze clés, relevée dans `src-tauri/pods/*.toml`

| `cle_heritee` | POD | format | reliure |
|---|---|---|---|
| `lulu` | `lulu` | `108x175` | `broche` |
| `bod` | `bod` | `135x215` | `broche` |
| `kdp-5x8` | `kdp` | `5x8` | `broche` |
| `kdp-55x85` | `kdp` | `55x85` | `broche` |
| `kdp-6x9` | `kdp` | `6x9` | `broche` |
| `coollibri-110x170` | `coollibri` | `110x170` | `broche` |
| `coollibri-148x210` | `coollibri` | `148x210` | `broche` |
| `coollibri-160x240` | `coollibri` | `160x240` | `broche` |
| `tbe-110x170` | `tbe` | `110x170` | `broche` |
| `tbe-120x180` | `tbe` | `120x180` | `broche` |
| `tbe-1485x210` | `tbe` | `1485x210` | `broche` |
| `bookvault-127x203` | `bookvault` | `127x203` | `broche` |
| `bookvault-129x198` | `bookvault` | `129x198` | `broche` |
| `bookvault-148x210` | `bookvault` | `148x210` | `broche` |

⚠ **La clé du POD TheBookEdition est `tbe`, pas `thebookedition`** (le nom du fichier est
`thebookedition.toml`). C'est exactement le genre d'écart qui produit « un nom de POD qui
se contredit d'une tâche à l'autre ».

La reliure est `broche` pour les quatorze — il n'y a aujourd'hui **aucune** autre reliure
dans les six fichiers. La migration n'a donc pas de choix à faire, ce qui confirme la
spec § 4 (« la conversion est totale et sans choix à faire »). La règle générale reste :
**la première reliure composable du POD** (celle qui porte une `geometrie`), que
`catalogue::aplatit` (`catalogue.rs:606`) applique déjà.

### `serde` peut-il lire les deux formes sans drapeau de version ?

Oui, avec `#[serde(untagged)]`. Testé :

```
neuf   -> Neuf { pod: "bod", format: "135x215", reliure: "broche", papier: "creme-90" }
ancien -> Ancien { provider: "bod", papier: "creme-90" }
fautif -> ERREUR : data did not match any variant of untagged enum DeuxFormes
```

**Le prix est le troisième cas, et il est cher.** Un `.ozalid` avec une faute de frappe ne
dit plus *ce qui* manque : `data did not match any variant`. Le message perd le nom du
champ, la ligne, et la raison. C'est en contradiction directe avec la discipline du dépôt —
`catalogue.rs` refuse en nommant le fichier, la ligne et ce qui manque (les trois refus du
module, `catalogue.rs:26-30`).

**Recommandation : ne pas prendre `untagged`.** Monter `VERSION` à 5 et ajouter une branche
dans `migre` coûte moins et dit mieux. Le dépôt le fait déjà deux fois.

## Verdict 4 — les noms de package

### D'où viennent les morceaux

`bod-135x215-broche-creme90` = `pod.cle` + `format.cle` + `reliure.cle` + `papier.cle`.
Trois sur quatre concordent avec les fichiers. Le quatrième non :

```
brut       : bod-135x215-broche-creme-90       ← ce que les clés donnent
sans tiret : bod-135x215-broche-creme90        ← ce que la spec § 4 écrit
```

**La clé du papier BoD est `creme-90`, avec un tiret** (`src-tauri/pods/bod.toml:35`). Pour
obtenir `creme90`, il faut un `.replace('-', "")` sur le seul segment papier — arbitraire,
et il rend `creme-90` et `cre-me90` indistinguables. **C'est une contradiction de la spec à
trancher dans le plan** : soit accepter `bod-135x215-broche-creme-90` (cinq segments
visibles, aucune transformation), soit changer la clé du papier en `creme90` dans
`bod.toml`.

### ⚠ Un trou de sécurité que le lot 2 crée

`est_un_nom` (`catalogue.rs:235`) ne s'applique **qu'à `cle_heritee`** (`catalogue.rs:302`).
Les quatre clés qui vont nommer le répertoire au lot 2 ne passent que par `cle_non_vide`.
Sonde exécutée sur la copie — trois `Pod::depuis_toml` avec un socle valide et une seule
clé de travers chacun :

```
pod.cle = "../evade"          -> ACCEPTÉ (cle = "../evade", papier = "creme-90", format = "135x215")
papier.cle = "../../ailleurs" -> ACCEPTÉ (cle = "essai", papier = "../../ailleurs", …)
format.cle = "C:nul*"         -> ACCEPTÉ (cle = "essai2", …, format = "C:nul*")
```

Aujourd'hui c'est inoffensif : seule `cle_heritee` nomme un répertoire, et elle est
contrainte. **Au lot 2, `cle_heritee` disparaît et ces quatre-là prennent sa place** : un
`<config>/pods/*.toml` déposé sur le poste pourrait faire écrire `package::assembler`
n'importe où. Le remède est d'un bloc : étendre le `est_un_nom` de `verifie` (`catalogue.rs:301-308`)
aux clés de POD, format, reliure et papier — et le lot 2 est le bon moment, puisque c'est
lui qui retire le garde-fou existant.

### `slug()` de `maquettes.rs` est-elle réutilisable telle quelle ?

`pub fn slug(nom: &str) -> Option<String>` (`maquettes.rs:405`). Sonde :

```
slug("creme-90")     = Some("creme-90")      ← le tiret interne survit
slug("Crème 90 g")   = Some("creme-90-g")
slug("5,5 × 8,5 po") = Some("5-5-8-5-po")
slug("../evade")     = Some("evade")         ← elle assainit bien
slug("C:nul*")       = Some("c-nul")
slug("«»")           = None
```

**Oui, elle est réutilisable et elle assainit correctement** — mais elle ne produit pas
`creme90` : elle ne supprime pas les tirets internes. Et un `Option` à traiter, là où
`est_un_nom` rend un `bool` et refuse à la lecture du fichier. **Refuser à la lecture vaut
mieux que slugifier au moment d'écrire** : le fichier fautif se nomme à la Livraison
(c'est le dernier commit du lot 1), et l'utilisateur voit quelle clé corriger, au lieu de
découvrir un répertoire au nom qu'il n'a pas écrit.

## Verdict 5 — ce qui a l'air simple et ne l'est pas

### 5a. Le littéral brut qui ne compile pas — le même qu'au lot précédent

En écrivant la sonde j'ai reproduit exactement la faute :

```
error: expected one of `)`, `,`, `.`, `?`, or an operator, found `"\n dos = { forme = "multiplie`
  --> examples/sonde.rs:27:18
   |
27 |   teinte = "#f7f0e0"
   |                    ^ expected one of `)`, `,`, `.`, `?`, or an operator
```

**Tout littéral brut Rust contenant un TOML de POD doit être `r##"…"##`**, parce que
`teinte = "#f7f0e0"` contient la séquence `"#` qui ferme un `r#"…"#`. Le
`catalogue.rs` existant le sait déjà pour ses constantes de test. Le plan doit l'écrire
noir sur blanc : chaque tâche qui pose un TOML d'essai avec une `teinte` est concernée.

### 5b. La mesure déplacée traverse jusqu'au JavaScript

`ProjetVue.livraison` (`commands.rs:132`) est **la `Livraison` du projet elle-même**, pas
une vue dédiée. Tout changement de forme arrive tel quel dans le front. Cinq lectures de
`compose` côté JS en dépendent : `app.js:229, 396, 531, 600, 622` et `couverture.js:687`.

**Une `LivraisonVue` qui recalcule `compose` sur chaque livrable depuis
`mesures[cle_gabarit]` rendrait le déplacement de la mesure entièrement invisible au
JavaScript.** Le seul changement côté front serait alors le renommage de l'identité — ce
qui divise par deux le travail sur les 9 fichiers de test JS. À arbitrer explicitement,
parce que ce n'est pas le choix par défaut.

### 5c. Deux emplacements possibles pour le même PDF d'intérieur

Le PDF d'intérieur ne dépend pas du papier : il est **identique** pour BoD crème et BoD
blanc. Mais :

- `composer` (`commands.rs:606`) écrit sous `sorties_dossier(o, &pr.cle)` ;
- `packager` (`commands.rs:1297`) écrit sous le même chemin, par livrable ;
- `vue` (`commands.rs:2015-2016`) **reconstruit** le chemin pour en faire un lien.

Si le répertoire de package suit le livrable (4 axes, spec § 4) et que la mesure suit le
gabarit (3 axes, spec § 5), il faut décider où `composer` écrit son PDF de travail. Les
deux choix se défendent ; ce qui ne se défend pas, c'est que `composer` et `vue` en
choisissent un chacun — le lien du pied pointerait alors vers un fichier absent, et le
commentaire de `interieur_pdf` (`commands.rs:1945-1947`) existe précisément pour prévenir
ça.

Corollaire pour le test « deux livrables du même gabarit ne déclenchent **qu'une**
composition » (spec § 9) : aujourd'hui `packager` appelle `package::assembler` par
destinataire, et `assembler` appelle `interieur::converge` (`package.rs:78`) sans
condition. Il n'y a **aucun** mécanisme de mémoïsation à étendre — il est à créer.

### 5d. `d.provider` est un identifiant de DOM

`livraison.js` fabrique `dest-papier-${d.provider}`, `dest-retirer-${d.provider}`,
`dest-${quoi}-${d.provider}` (lignes 73, 88, 97, 131-139). L'identité à quatre axes doit
donner une chaîne :
- **unique** — c'est acquis, c'est le sens de l'identité à quatre ;
- **valide en `id` HTML** — acquis si les quatre clés sont contraintes (verdict 4) ;
- **stable** — donc fabriquée par le Rust et servie au front, pas recomposée en JS à deux
  endroits qui divergeront.

### 5e. Le COOKBOOK ment déjà

`docs/COOKBOOK.md:8` : « `src-tauri/src/providers.rs` **fait foi** ». Ce fichier n'existe
plus depuis le lot 1. Voir aussi les lignes 68 et 76 (« table `providers` », « la compléter
dans `providers.rs` »). La spec range le COOKBOOK au lot 4 — c'est donc une dette assumée,
pas un oubli, mais elle est là et elle grandit.

### 5f. `Mesure` n'est pas `Copy`

`projet.rs:258-262` le dit et l'explique. Toute lecture derrière une référence demande un
`.clone()` (`projet.rs:1500-1502` en donne l'exemple). Une esquisse de plan qui écrirait
`let m = livraison.mesures[cle];` ne compilerait pas.

### 5g. Le test d'intégration séparé

`src-tauri/tests/catalogue_initialise.rs` — un seul test, hors de la lib, parce que
`initialiser` pose un `OnceLock` une fois par processus. Si le lot 2 remplace `PLATS` par
`PODS`, **c'est ce fichier-là qu'il faut penser à changer**, et il n'apparaît dans aucun
`grep` sur `src/`.

---

# 8. Ce qui a bougé entre `11c6f46` et `13bdd13`

Vérifié fichier par fichier contre le HEAD actuel. Cinq fichiers ont changé :
`catalogue.rs`, `interieur.rs`, `examples/temoin.rs`, `src/livraison.js`,
`tests/contrats.test.js`. `commands.rs`, `projet.rs` et `package.rs` : **aucun
changement**.

## 8a. Fait neuf n°1 — `Provider` a perdu trois champs

`dadec0f` retire `corps_pt`, `interligne` et `folio_pt` de `catalogue::Provider` et de
`aplatit` ; ils deviennent des constantes de `interieur`. **`Provider` n'a plus que onze
champs** : `cle`, `libelle`, `format`, `marge_haut`, `marge_bas`, `exterieur`,
`gouttieres`, `fond_perdu`, `pages_min`, `pages_max`, `papiers`.

Conséquence pour le verdict 1c : le `Provider` fabriqué à la volée est d'autant moins cher
à cloner. La liste des champs que le lot 2 doit savoir reconstituer depuis les cinq axes
est celle-ci et pas une autre.

## 8b. Fait neuf n°2 — ⚠ un test d'ancrage tout neuf fige les quatorze clés plates

`13bdd13` ajoute `catalogue.rs:1608` :

```rust
fn la_liste_des_prestataires_garde_ses_quatorze_libelles_dans_l_ordre() {
    let vue: Vec<(&str, &str)> = providers().iter().map(|p| (p.cle.as_str(), p.libelle.as_str())).collect();
    assert_eq!(vue, [
        ("lulu", "Lulu — poche 108 × 175"),
        ("bod", "BoD — 13,5 × 21,5 cm"),
        ("kdp-5x8", "Amazon KDP — 5 × 8 po"),
        …
        ("bookvault-148x210", "Bookvault — A5 148 × 210"),
    ]);
}
```

**C'est le test qui casse le plus fort au lot 2**, et il n'était pas là quand j'ai fait mon
inventaire du § 6 : il fige d'un bloc les quatorze **clés plates**, leurs **libellés** et
leur **ordre** — c'est-à-dire exactement ce que le lot 2 supprime. Son commentaire dit
pourquoi il existe (« c'est un choix de prestataire que l'utilisateur fait dans cette
liste, à la lecture de ces libellés »).

**Le plan doit le nommer explicitement** et dire lequel des deux : il devient l'ancrage des
livrables (quatorze combinaisons POD × format × reliure × papier par défaut, avec leurs
libellés), ou il est retiré délibérément parce que le lot 3 le remplace par la cascade.
Le laisser mourir en silence, c'est le douzième test perdu du lot précédent.

Ce test donne aussi, gratuitement, **la table des libellés attendus**, utile au plan :
`Lulu — poche 108 × 175`, `BoD — 13,5 × 21,5 cm`, `Amazon KDP — 5 × 8 po`,
`Amazon KDP — 5,5 × 8,5 po`, `Amazon KDP — 6 × 9 po`, `CoolLibri — 11 × 17 cm`,
`CoolLibri — A5`, `CoolLibri — 16 × 24 cm`, `TheBookEdition — Poche 11 × 17`,
`TheBookEdition — Manga 12 × 18`, `TheBookEdition — A5 14,8 × 21`,
`Bookvault — Novel 127 × 203`, `Bookvault — B Format 129 × 198`, `Bookvault — A5 148 × 210`.

## 8c. Décalages de lignes à reporter

| ce que je cite | ligne à `11c6f46` | ligne au HEAD |
|---|---|---|
| `interieur::converge` | 113 | **112** |
| `interieur::assemble` | 182 | **179** |
| `interieur::source` | 428 | **425** |
| `interieur::source_ebook` | 447 | **444** |
| `livraison.js` `dest-papier-${d.provider}` | 73 | **75** |
| `livraison.js` `dest-retirer-${d.provider}` | 97 | **99** |

Inchangés et vérifiés au HEAD : `interieur.rs:238` (`pr.cle` dans la source Typst),
`catalogue.rs:235` (`est_un_nom`), `catalogue.rs:266` (`verifie`), `catalogue.rs:534`
(`FOURNIS`), `catalogue.rs:606` (`aplatit`), `catalogue.rs:650/655`
(`providers` / `provider`), `package.rs:54` (`nom`), et les treize lignes de `commands.rs`
citées au § 3 et § 4.

## 8d. Ce qui n'a pas changé et qu'il fallait revérifier

- **`est_un_nom` n'est appliqué qu'à `cle_heritee`** (`catalogue.rs:303`, un seul site).
  Le trou du verdict 4 est intact.
- **Le témoin est inchangé** : `PROVIDER = "bod"` (`examples/temoin.rs:26`),
  `PAGES_ATTENDUES = 98` (ligne 34). Le déplacement du corps et de l'interligne n'a donc
  pas bougé la pagination — ce que mon exécution du témoin confirme par ailleurs.
- `tests/contrats.test.js` gagne huit lignes sur le découpage du chemin d'un fichier
  refusé : sans rapport avec le lot 2, mais c'est un fichier de la liste du § 6.

---

## Ce que je n'ai pas pu établir

- **Le compte exact des tests JS qui échouent.** Le renommage naïf fait *boucler* la suite
  au lieu de la faire échouer (§ 6). Le fait — la boucle — est plus utile que le compte,
  mais le compte reste inconnu.
- **`cargo clippy`** n'a pas été passé sur les esquisses : elles portent des `todo!()` et
  des `pub` de circonstance, clippy n'y dirait rien de transposable.
- **Le comportement à l'écran** : aucune fenêtre n'a été lancée.
