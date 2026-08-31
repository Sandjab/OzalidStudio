# Une version portable pour Windows

> État : spec de chantier, écrite le 31/08/2026 sur le dépôt en version 1.1.0.
> Les renvois de ligne pointent le code de ce jour.

## Objectif

L'application ne se distribue aujourd'hui sous Windows que par un installeur NSIS,
produit sur tag par `.github/workflows/windows.yml`. Il installe sans droits
administrateur, mais il installe : il écrit dans `%LOCALAPPDATA%`, pose une entrée de
désinstallation, et laisse les réglages de l'utilisateur sur la machine.

Ce chantier ajoute une **seconde forme de livraison** : une archive `.zip` qu'on déplie
où l'on veut — un disque, un dossier partagé, une clé USB — et qui garde ses réglages
dans le dossier déplié plutôt que sur le poste. Rien n'est installé, rien n'est laissé
derrière, et la même clé rouvre ses maquettes sur un autre ordinateur.

L'installeur reste la voie normale. Le portable est la voie de celui qui ne peut pas
installer, ou qui ne veut pas.

## Décisions de cadrage (brainstorming du 31/08)

- **Un marqueur livré dans l'archive**, et lui seul, dit que l'application est portable :
  un fichier vide portant le nom de l'exécutable et l'extension `.portable`. L'installeur
  ne le pose jamais — aucune installation existante ne peut basculer par accident.
- **Les données vont dans un sous-dossier `donnees/`** à côté de l'exécutable, non à
  plat : ce qui vient de l'archive et ce que l'usage a produit restent distinguables à
  l'œil, et l'un se sauvegarde sans l'autre.
- **Marqueur présent mais dossier non inscriptible : on le dit, et on reste portable.**
  L'application démarre, lit ce qu'elle trouve, et annonce que rien ne sera enregistré.
  Elle ne se rabat pas en silence sur `%APPDATA%` — le mode portable existe précisément
  pour que les réglages ne partent pas sur la machine hôte.
- **La preuve passe par un drapeau de diagnostic** sur l'exécutable, que la CI interroge
  sur l'archive réellement produite. Pas de session graphique, pas de délai deviné.
- **Les récents sur clé USB ne sont pas traités ici** (§ 9), ni l'absence de WebView2
  sur un Windows ancien (§ 9). Documentés, non corrigés.
- **`tauri build` ne produit pas l'archive** : Tauri 2 n'offre sous Windows que les
  cibles `nsis` et `msi`. L'archive est assemblée par un script du dépôt, comme le
  sidecar et les polices le sont déjà.

## 1. Ce que le mode portable change, et ce qu'il ne change pas

Trois choses seulement vivent hors des projets, et elles descendent toutes du même
`config: &Path` :

| Ce qui vit là | Qui le lit et l'écrit |
|---|---|
| `preferences.toml` — récents, accès de diffusion, gabarit par défaut | `preferences.rs` |
| `maquettes/` — les maquettes personnalisées | `maquettes.rs:143` |
| les surcharges de catalogue (`.toml` d'imprimeur) | `catalogue.rs:1129` |

C'est donc **un seul chemin** à dévier, et non trois politiques à tenir cohérentes.

Ce qui ne change pas : le sidecar Typst et les polices sont **déjà** cherchés à côté de
l'exécutable (`commands.rs::binaire_typst`, `commands.rs::typst`). Une archive dépliée à
plat compose donc sans qu'une ligne bouge — c'est ce que le job `verifier` de la CI
établit déjà pour l'installeur, au même emplacement.

Ce qui ne change pas non plus : les projets. Un `.ozalid` est un fichier que
l'utilisateur range où il veut ; le mode portable ne le déplace pas et ne le suppose
nulle part.

## 2. Le marqueur

À côté de l'exécutable, un fichier **vide** dont le nom est celui de l'exécutable privé
de son extension, suivi de `.portable` :

```
Ozalid Studio 1.1.0/
├── ozalid-studio.exe
├── ozalid-studio.portable      ← le marqueur, vide
├── typst.exe
├── fonts/
│   └── *.ttf
└── donnees/                    ← créé au premier lancement
```

Pourquoi ce nom plutôt qu'un `portable.txt` fixe : il se lit sans documentation, il se
supprime pour revenir au comportement installé, et il suit l'exécutable si celui-ci est
un jour renommé. Son contenu n'est jamais lu — c'est sa présence qui parle. Un contenu
qui dirait le chemin des données a été écarté au cadrage : c'est une option de
configuration à spécifier et valider, pour un besoin non exprimé.

Le marqueur n'a pas de plateforme : le code qui le cherche est le même partout. Seule
l'archive Windows en pose un. Un développeur qui en dépose un dans `target/debug/`
obtient une application portable en développement, ce qui est le moyen le plus court de
la regarder à l'œil.

## 3. `emplacement.rs` : la décision, prise une fois

Un module neuf, dont la seule question est : *où l'application écrit-elle ce qui
n'appartient pas à un livre ?*

```rust
pub enum Mode {
    /// Pas de marqueur : le répertoire de configuration du système.
    Installe,
    /// Marqueur posé, et `donnees/` accepte l'écriture.
    Portable,
    /// Marqueur posé, mais rien ne pourra être enregistré.
    PortableLectureSeule,
}

pub struct Emplacement {
    /// Le répertoire de configuration, quel que soit le mode. `None` si même le
    /// système n'en propose pas — l'application démarre alors sur les défauts.
    pub racine: Option<PathBuf>,
    pub mode: Mode,
}

/// `systeme` est ce que Tauri propose (`app_config_dir()`), passé plutôt que lu :
/// c'est ce qui rend ce module testable sans Tauri ni Windows.
pub fn resoudre(systeme: Option<PathBuf>) -> Emplacement;
```

`resoudre` procède ainsi :

1. `std::env::current_exe()`, puis son répertoire. Indisponible → `Installe`, racine
   `systeme`. C'est le cas d'un environnement qui n'a pas d'exécutable au sens usuel ;
   il ne doit pas empêcher le démarrage.
2. Le marqueur n'est pas là → `Installe`, racine `systeme`. **C'est le chemin de toutes
   les installations existantes, et il doit rester exactement celui d'aujourd'hui.**
3. Le marqueur est là → racine `<dossier de l'exe>/donnees`. On tente
   `create_dir_all`, puis l'écriture et l'effacement d'un fichier témoin. Succès →
   `Portable`. Échec → `PortableLectureSeule`, **racine servie quand même** : lire ce
   qui est déjà là reste possible et utile.

Le test d'inscriptibilité écrit vraiment plutôt que d'interroger des permissions : sous
Windows, un attribut de fichier ne dit pas ce qu'un partage réseau ou une stratégie de
groupe autorisera à l'écriture. La seule réponse fiable est la tentative.

`lib.rs::run` résout **en première ligne du `setup`**, avant `catalogue::initialiser` —
dont le commentaire existant impose déjà cette place, et qui consomme précisément un
`Option<&Path>`. Le résultat est posé en état Tauri managé, à côté de `CatalogueRefus`.

## 4. Les trois appelants

Ils cessent d'appeler `app.path().app_config_dir()` et lisent l'état managé :

| Fichier | Aujourd'hui | Demain |
|---|---|---|
| `lib.rs:41` | `app.path().app_config_dir().ok().as_deref()` | la racine de l'`Emplacement` résolu juste avant |
| `commands.rs:2609` (`fn config`) | idem | `app.state::<Emplacement>().racine.clone()` |
| `menu.rs:163` (`liste_recents`) | idem | idem |

Après ce chantier, `app_config_dir()` n'apparaît **plus qu'une fois** dans le dépôt, dans
le `setup`. C'est la propriété qui empêche un quatrième appelant de rouvrir un chemin
parallèle sans qu'on le voie, et une vérification de revue tient en un `grep`.

## 5. Ce que l'interface en dit

Le mode lecture seule suit le chemin déjà tracé par les refus de catalogue, sans rien
inventer :

- état managé au `setup` (comme `commands.rs:29`, `CatalogueRefus`) ;
- une commande qui le rend (comme `commands.rs:282`, `catalogue_refus`) ;
- un bandeau au front (comme `livraison.js:21`).

Le bandeau va sur l'**accueil** (`index.html:43`), au pied du bloc Projet : c'est l'écran
où l'on arrive, et l'avertissement concerne tout ce qui suit, pas une étape. Classe
`note alerte`, celle que `livrablesElagues` (`index.html:386`) emploie déjà pour ce
registre.

Il ne s'affiche **que** sous `PortableLectureSeule`. Ni `Installe` ni `Portable` ne
disent quoi que ce soit : une application qui fonctionne n'a pas à s'expliquer.

Sous `Portable`, en revanche, rien n'est masqué non plus — le drapeau du § 6 répond à qui
demande.

## 6. Le drapeau de diagnostic

```
ozalid-studio.exe --emplacement <fichier>
```

écrit dans `<fichier>` le mode et la racine résolue, puis sort sans ouvrir de fenêtre.

**Il écrit dans un fichier, et non sur la sortie standard**, pour une raison qui n'est
pas un détail : `main.rs:2` pose `windows_subsystem = "windows"` en release, donc
l'exécutable n'a aucune console rattachée. Un `println!` ne serait lu ni par la CI en
PowerShell, ni par l'utilisateur qui l'appelle depuis un terminal. Rattacher la console
du parent demanderait une dépendance Windows ; passer le chemin de sortie en argument
n'en demande aucune, et c'est déjà la façon dont `examples/temoin.rs:62` prend la sienne.

Le traitement se fait dans `run()`, avant que Tauri ne construise quoi que ce soit. Un
argument inconnu est ignoré, comme aujourd'hui : ce drapeau ne fait pas de cet
exécutable une commande.

Il sert deux publics : la CI, qui en fait sa preuve (§ 8), et l'utilisateur qui ne
retrouve pas ses maquettes.

## 7. L'archive : `outils/portable.sh`

Un script, à côté de `typst.sh` et `polices.sh`, dont c'est la convention du dépôt. La CI
l'appelle ; **le développeur aussi**, et c'est le point : sans exécution locale possible,
personne n'ouvrirait le portable avant le premier utilisateur.

Il assemble, depuis un `cargo build --release` déjà fait :

| Source | Destination dans l'archive |
|---|---|
| `src-tauri/target/release/ozalid-studio.exe` | à la racine du dossier |
| `src-tauri/binaries/typst-<triplet>.exe` | `typst.exe` — **renommé**, comme le fait le bundle Tauri pour un `externalBin`, et comme la CI le vérifie déjà à l'installation |
| `src-tauri/fonts/*` | `fonts/` |
| — | `ozalid-studio.portable`, créé vide |

Tout va dans **un dossier de tête** nommé d'après le produit et sa version, pour qu'un
dépliage ne disperse pas trente fichiers dans le dossier des téléchargements. La version
se lit dans `tauri.conf.json`, comme le job `publier` le fait déjà pour contrôler le tag.

Nom de l'archive : `ozalid-studio_<version>_x64-portable.zip`, qui fait écho au
`Ozalid Studio_<version>_x64-setup.exe` de NSIS.

`donnees/` n'est **pas** dans l'archive : il naît au premier lancement. Une archive qui
le contiendrait vide ferait croire à un dossier livré, donc à un dossier qu'on peut
écraser en mettant à jour.

## 8. La CI

Dans le job `publier`, après la construction de l'installeur et avant la release :

1. `outils/portable.sh`. L'exécutable de release existe déjà : `tauri build --bundles
   nsis` vient de le produire, le script n'a rien à recompiler.
2. **La preuve** : déplier l'archive dans un dossier temporaire, lancer
   `ozalid-studio.exe --emplacement rapport.txt`, et vérifier que le rapport annonce le
   mode `Portable` et une racine égale à `<dépliage>/donnees`. Un chemin qui pointerait
   `%APPDATA%` fait échouer le job.
3. Vérifier l'arborescence dépliée comme elle l'est déjà après l'installation
   silencieuse : `typst.exe` à côté de l'exe, au moins un `.ttf` dans `fonts/`.
4. Joindre l'archive à la même release draft que l'installeur.

L'étape 2 est ce qui distingue ce chantier d'une promesse : elle interroge le **binaire
livré**, par le code de production, et non un test qui simulerait la même logique.

Le job `verifier` n'est pas touché.

## 9. Risques et limites

**WebView2 absent.** Une archive n'installe pas de runtime. Sur Windows 11 et sur un
Windows 10 à jour, WebView2 est préinstallé ; sur un poste ancien, l'installeur NSIS
sait le télécharger et le portable ne le peut pas. L'application ne s'ouvrira pas, et le
message du système ne dira pas pourquoi. **À écrire dans les notes de release**, avec le
renvoi vers l'installeur comme solution.

**La marque de provenance.** Un `.zip` téléchargé porte sous Windows une marque qui se
propage aux fichiers extraits ; SmartScreen avertira au lancement, et certaines
stratégies bloquent purement. La marche à suivre — propriétés de l'archive, « Débloquer »,
puis extraire — est de même nature que l'avertissement SmartScreen déjà documenté pour
l'installeur, et se documente au même endroit.

**Les récents sur clé USB.** Les récents sont des chemins absolus (`preferences.rs`), et
la lettre de lecteur change d'un poste à l'autre. Sur le poste suivant, ils ne répondent
plus. La dégradation est douce — `recents_existants()` les écarte déjà, rien ne plante,
la liste se repeuple au premier projet rouvert — et le correctif toucherait le format de
`preferences.toml`, hors du sujet de ce chantier. **Limite documentée, dette assumée.**

**Deux copies des réglages.** Un utilisateur qui a l'installeur *et* le portable a deux
jeux de préférences, sans passerelle. C'est la définition du portable, pas un défaut ;
il faut néanmoins que la documentation le dise, sans quoi « mes maquettes ont disparu »
est la première question.

**Le risque à ne pas prendre.** Une erreur dans `resoudre` qui rendrait `Portable` sans
marqueur enverrait les réglages d'une installation existante dans son dossier
d'installation. C'est le seul scénario destructeur du chantier, et c'est celui que le
test du § 10 doit voir échouer en premier.

## 10. Vérification

### Ce que les tests doivent tenir

Dans `emplacement.rs`, sur `tempfile`, sans Tauri ni Windows :

1. **Sans marqueur, la racine est celle du système.** La régression protégée est la
   seule destructrice du chantier (§ 9).
2. **Avec marqueur, la racine est `<exe>/donnees`**, et le dossier est créé.
3. **Le marqueur porte le nom de l'exécutable** : un `autre.portable` posé à côté ne
   déclenche rien. Sans ce test, un `*.portable` accidentel suffirait.
4. **Dossier non inscriptible → `PortableLectureSeule`, racine servie quand même.** La
   racine est ce qui distingue la décision retenue du repli silencieux écarté au
   cadrage.

Dans `preferences.rs` et `maquettes.rs` : rien de neuf. Ils reçoivent déjà un `&Path` et
ne savent pas d'où il vient — c'est ce qui fait tenir ce chantier en un point.

### Le rouge exigé

Chaque test ci-dessus doit avoir été **vu échouer** avant que le code ne le satisfasse
(CLAUDE.md, § Vérifications avant commit). Pour ceux qui ne peuvent pas naître rouges,
la mutation ciblée est nommée d'avance :

| Test | Mutation qui doit le faire tomber |
|---|---|
| 1 | supprimer la vérification de présence du marqueur |
| 3 | comparer sur l'extension seule au lieu du nom complet |
| 4 | retomber sur `systeme` au lieu de servir la racine |

### Le témoin

`cargo run --example temoin` : le compte de pages ne doit pas bouger. Aucun code de
composition n'est touché, et un écart signalerait le piège déjà documenté des ressources
embarquées — `touch src-tauri/{pods,maquettes} src-tauri/src/lib.rs` avant de conclure.

### À l'œil, ce que la CI ne prouve pas

La CI établit que l'archive résout le bon chemin. Elle n'ouvre pas la fenêtre. Restent à
faire une fois, sur une vraie clé :

- déplier l'archive sur une clé USB, lancer, enregistrer une maquette, vérifier qu'elle
  est bien sous `donnees/maquettes/` et **nulle part dans `%APPDATA%`** ;
- débrancher, rebrancher sur un autre poste, retrouver la maquette ;
- déplier sur un support en lecture seule et lire le bandeau de l'accueil ;
- composer un livre depuis le portable — le sidecar et les polices sont censés suivre,
  et c'est le seul endroit où on le verra vraiment.

## 11. Documentation

- `README.md`, section **Windows** (`README.md:38`) : l'archive à côté de l'installeur,
  ce qu'elle apporte, le dossier `donnees/`, et les deux réserves — WebView2, marque de
  provenance.
- `README.md:42` mentionne aujourd'hui `%LOCALAPPDATA%\Ozalid Studio` comme l'endroit des
  réglages : la phrase devient conditionnelle au mode.
- Les notes de la release draft (`.github/workflows/windows.yml`, étape *Release draft*)
  gagnent le paragraphe WebView2.

## Hors périmètre

- **Un portable pour macOS.** Un `.app` se déplace déjà ; le besoin n'est pas le même, et
  le marqueur devrait vivre dans le bundle. Le code de `resoudre` n'a pourtant aucun
  `cfg` : le jour où on le voudra, il n'y aura qu'une archive à produire.
- **Les récents relatifs au volume** (§ 9).
- **La signature de code**, déjà hors périmètre du chantier CI et inchangée ici.
- **Une migration entre installé et portable.** Copier `donnees/` à la main est un geste
  d'utilisateur ; l'automatiser demanderait de choisir quoi faire des conflits.
- **Une mise à jour en place du portable.** On redéplie l'archive à côté et on recopie
  `donnees/`. C'est précisément ce que le sous-dossier rend possible.
