# Reconnaissance — Livraison refondue, lot 1 (le modèle)

Date : 2026-08-29
Spec : `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md`

Ce que le lot 1 suppose, vérifié dans le code avant d'écrire le plan. Chaque verdict porte
le fichier et la ligne qui le fondent.

## 1. Le champ neuf ne casse aucun `.ozalid`

**1a.** `Livrable` (`projet.rs:300`) porte quatre champs, dont `fabrication` en
`#[serde(flatten)]` et trois optionnels écrits
`#[serde(default, skip_serializing_if = "Option::is_none")]`. Un cinquième champ optionnel
suit exactement ce moule.

**1b.** `VERSION` ne bouge pas. Le précédent est écrit noir sur blanc pour `livraison` et
`envois` (`projet.rs:578` et `583`) : « Facultative : un `.ozalid` écrit avant elle s'ouvre
sans rien dire […] `VERSION` ne bouge donc pas. » Un livrable sans état de génération
s'ouvre en *jamais généré* par le `Default` de son type.

## 2. Le hachage : celui qui existe ne convient pas

**2a.** Le dépôt a déjà une fonction `empreinte(&str) -> String` (`commands.rs:2058`), bâtie
sur `DefaultHasher`, pour nommer le répertoire des rendus d'un fond. Son commentaire dit
pourquoi elle suffit là : « ce n'est pas un contrôle d'intégrité, seulement un nom qui change
quand la source change. Une collision coûterait des vignettes périmées, pas un mauvais
tirage. »

**2b. Elle ne convient pas à cet usage-ci, et c'est le verdict qui coûte le plus cher à
ignorer.** `DefaultHasher` n'est pas garanti stable d'une version de Rust à l'autre — la
bibliothèque standard le documente. Nommer un répertoire de vignettes s'en moque : une valeur
qui change fabrique un répertoire neuf et l'on recalcule. Mais une empreinte **persistée dans
le `.ozalid` et comparée après une mise à jour du binaire** marquerait, elle, tous les
packages du projet comme périmés d'un coup, sans que rien à l'écran puisse l'expliquer.

**Verdict** : le lot 1 écrit sa propre empreinte, déterministe à demeure (FNV-1a, six
lignes), et son commentaire doit dire pourquoi elle ne réutilise pas celle de `commands.rs`.
Ne pas fusionner les deux : elles n'ont ni la même exigence ni la même durée de vie.

## 3. Ce qui entre dans quelle empreinte

**3a. `projet.images` sert la couverture, et elle seule.** `ecrire_images` n'a qu'un
appelant, `package.rs:443`, dans `assembler` ; elle range ses entrées en première ou en
quatrième selon `sert_la_quatrieme`.

**3b. `projet.polices` est la main de l'auteur, pas une police de composition.**
`ecrire_polices` n'a qu'un appelant, `package.rs:739`, dans `assembler_envois` — le chemin
des exemplaires dédicacés. Hors périmètre des deux empreintes.

**3c. `livre` entre dans les *deux* empreintes.** Il fait la page de titre à l'intérieur et
alimente les jetons `%TITRE%`, `%AUTEUR%`, `%ISBN%` que la couverture cite. L'oublier d'un
côté laisserait une moitié du livre à jour et l'autre fausse.

**3d. L'empreinte de couverture doit porter le dos effectif**, donc le papier *et* la
pagination. Sans quoi un changement de police — qui repagine, donc change le dos — laisserait
la planche marquée à jour alors que son dos a bougé. C'est le risque nommé au § 8 de la spec.

## 4. Le précédent d'empreinte existe déjà dans le modèle

**4a.** `Resolu::empreinte()` (`catalogue.rs:1079`) rend l'empreinte géométrique d'un
gabarit — format, marges, gouttières — et `Mesure.empreinte` (`projet.rs:356`) la persiste
puis la compare à l'ouverture, par `normalise`. Le dispositif du lot 1 n'invente donc pas un
mécanisme : il en généralise un qui tourne déjà. L'empreinte d'intérieur réutilise cette
valeur telle quelle pour sa part « gabarit ».

## 5. Le coût

**5a.** Hacher le manuscrit du témoin (≈ 300 Ko) coûte quelques dixièmes de milliseconde,
contre plusieurs secondes de composition. Calculer les deux empreintes à chaque vue est sans
effet mesurable, et dispense d'un cache — donc d'une invalidation à tenir juste, ce que
`commands.rs:2068` refuse déjà pour les vignettes d'envoi et pour la même raison.

## 6. Où loger le code

**6a.** `projet.rs` fait 2983 lignes, `catalogue.rs` 3058, `interieur.rs` 3240 : de gros
modules sont la norme du dépôt, mais il porte aussi des modules courts et mono-sujet —
`detourage.rs` (264), `police.rs` (348), `ebook.rs` (357). Un module `empreinte.rs` de la
même taille est dans la manière de la maison, et donne une frontière éprouvable sans toucher
au disque ni à Typst — le parti déjà pris pour `dossiers_d_envoi` (`package.rs`) et pour
`meme_gabarit`.

**Verdict** : `src-tauri/src/empreinte.rs`, déclaré dans `lib.rs` entre `ebook` et `envoi`
(la liste est alphabétique).
