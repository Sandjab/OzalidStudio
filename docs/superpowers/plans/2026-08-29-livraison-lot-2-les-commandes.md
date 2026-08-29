# Livraison refondue, lot 2 — les commandes

> **Pour un exécutant agentique :** SOUS-COMPÉTENCE REQUISE : `superpowers:subagent-driven-development`
> (recommandé) ou `superpowers:executing-plans` pour exécuter ce plan tâche par tâche. Les
> étapes sont des cases à cocher (`- [x]`).

**But :** les quatre verbes d'un livrable — générer, remplacer, régénérer, supprimer — chacun
posant l'état que le lot 1 a rendu calculable, avec un seul chemin de composition et un
intérieur qui se réutilise **depuis le disque**. L'écran actuel continue de tourner : le front
n'est pas touché.

**Architecture :** le corps de `packager` sort en une fonction libre `composer_lot`, que les
quatre verbes appellent avec une cible ou avec toutes — c'est le « un seul jeu de garanties »
du § 4 de la spec. `package::lot` apprend à amorcer sa mémoïsation avec l'intérieur qu'un
autre livrable a déjà sur le disque, **les cibles de la passe exclues du vivier**
(reconnaissance 4d). La suppression sélective et le montage des empreintes sortent en
fonctions libres, éprouvables sans `State` et sans Typst — la manière déjà prise pour
`refuse_doublon`, `reglage_refuse`, `meme_gabarit` et `dossiers_d_envoi`.

**Pile :** Rust 2021, `serde`, `tempfile` (déjà en dépendance de développement). Tests :
`cargo test` depuis `src-tauri/`, `cargo test -- --ignored` pour ceux qui lancent Typst,
`cargo run --example temoin` comme témoin.

**Spec :** `docs/superpowers/specs/2026-08-29-livraison-refondue-design.md` (§ 3, 4, 6).
**Reconnaissance :** `docs/superpowers/2026-08-29-reconnaissance-livraison-lot-2.md` — les
verdicts cités ici (1a à 8) y sont, chacun appuyé sur un fichier et une ligne.

---

## Décisions arbitrées (29/08) — ne pas les rouvrir

Proposées à la lecture de la reconnaissance, non contestées :

1. **`livrable_supprimer` refuse de supprimer le dernier livrable**, comme `livrable_retirer`
   le fait : c'est lui qui donne le format sous lequel on regarde la couverture. Aucun fichier
   n'est effacé quand ce refus tombe.
2. **Remplacer garde le rang quand le POD ne change pas, et pousse en queue quand il change.**
   C'est la seule lecture qui satisfait à la fois « à sa place » (§ 6) et « en queue de son
   nouveau groupe » (§ 3).
3. **La finition n'entre dans aucune empreinte** (reconnaissance 8) : elle ne fabrique aucun
   octet du PDF, et le seul chemin qui la change recompose de toute façon.
4. **`Etat` ne descend pas encore dans la vue** : `livraison_vue` ne voit que la `Livraison`
   quand `empreinte::etat` réclame le `Projet` entier. C'est le lot 3, avec le reste de l'écran.
5. **Les trois anciennes commandes restent en place.** `livrable_ajouter`, `livrable_regler`
   et `livrable_retirer` ne disparaissent qu'au lot 3, quand l'écran cessera de les appeler.

## Contraintes globales

- **Français** dans les commentaires, les messages et les commits ; termes techniques anglais
  conservés tels quels (`chunk`, `viewport`, `canvas`).
- **Aucun test neuf ne compte s'il n'a pas été vu échouer.** TDD strict, ou mutation ciblée.
- `VERSION` du `.ozalid` **ne change pas** : le lot 1 a posé le champ, ce lot ne fait que
  l'écrire.
- Le témoin doit valoir **le même compte de pages qu'avant le lot** — 98 / 118 / 100.
- Le front (`src/*.js`) n'est pas touché, ni les tests `node --test`.

## Avant chaque commit

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test
cd .. && node --test tests/*.test.js
cd src-tauri && cargo run --example temoin     # dès qu'un fichier de src-tauri/ a bougé
```

`clippy` est rouge sur la baseline depuis rustc 1.98 — `police.rs:123` et
`examples/packager.rs:32`, lint `chunks_exact_to_as_chunks`. Ce sont les deux seuls
avertissements admis ; tout autre est de votre fait.

Baseline relevée le 29/08 avant d'écrire ce plan : `cargo test` **610 passés, 0 échec,
9 ignorés**.

## Ce que la spec § 9 réclame et que le lot 1 tient déjà

Quatre des huit tests de la liste de vérification sont écrits et verts depuis le lot 1 —
`empreinte.rs` et `projet.rs`. **Ne pas les réécrire** : les retrouver, et vérifier qu'ils
tiennent toujours après ce lot.

| ce que la spec réclame | où c'est déjà tenu |
|---|---|
| une couverture retouchée périme la couverture, et elle seule ; l'inverse aussi | `empreinte.rs`, tests de `Perime` |
| un envoi ajouté ou une épreuve tirée ne périment rien | `empreinte.rs` |
| un échec retenu ne se compare pas | `empreinte.rs` |
| un `.ozalid` d'avant s'ouvre en *jamais généré*, ses relevés intacts | `projet.rs` |

Les quatre autres sont les tâches 2, 3, 5 de ce plan.

## Structure des fichiers

| fichier | rôle |
|---|---|
| `src-tauri/src/package.rs` | **modifié** — l'amorce depuis le disque, la suppression sélective |
| `src-tauri/src/commands.rs` | **modifié** — `retenir`, `composer_lot`, les quatre commandes |
| `src-tauri/src/lib.rs` | **modifié** — les quatre commandes dans l'`invoke_handler` |

Aucun fichier créé : ce lot pose des fonctions dans les deux modules qui portent déjà la
composition et les commandes. Un module neuf séparerait des verbes de leur seul appelant.

---

### Tâche 1 : L'ordre qui rend l'empreinte juste

`empreinte::couverture` lit la pagination retenue (`empreinte.rs:64`) ; retenir la mesure
**après** avoir empreint ferait naître chaque package périmé sur sa couverture — c'est-à-dire
sur son dos. L'ordre est enfermé dans une fonction, avec le test qui l'attrape.

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (fonction neuve avant `packager`, et la boucle
  `commands.rs:1656-1670` remplacée par un appel)

**Interfaces :**
- Consomme : `crate::empreinte::{interieur, couverture}`, `projet::Generation`,
  `Livraison::retenir_mesure`.
- Produit : `fn retenir(projet: &mut Projet, cle: &str, issue: Result<&package::Package, String>)`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/commands.rs`, sous `#[cfg(test)] mod tests` :

```rust
/// Un package d'essai : seuls la pagination et les mesures comptent ici.
fn package_d_essai(cle: &str, pages: u32) -> package::Package {
    package::Package {
        cle: cle.into(),
        libelle: "Essai".into(),
        papier: "Crème".into(),
        pages,
        gouttiere: 14.0,
        blanche: false,
        dos: 16.0,
        dos_requis: None,
        fond_perdu: 3.0,
        planche: (300.0, 200.0),
        chemins: vec![],
        vignette: String::new(),
        polices_introuvables: vec![],
        avertissements: vec![],
        interieur_partage: false,
    }
}

/// **L'ordre décide de la justesse.** L'empreinte de couverture lit la pagination retenue :
/// empreindre avant de retenir la mesure la daterait de la composition précédente, et le
/// package naîtrait périmé sur son dos à la seconde même. Le test le prouve en faisant bouger
/// la pagination — 98 avant, 120 après.
#[test]
fn la_mesure_est_retenue_avant_que_les_empreintes_ne_soient_prises() {
    let mut o = ouvert_neuf();
    let l = o.projet.meta.livraison.livrables[0].clone();
    let gabarit = l.fabrication.cle_gabarit();
    o.projet.meta.livraison.retenir_mesure(
        &gabarit,
        Mesure {
            pages: 98,
            gouttiere: 14.0,
            blanche: false,
            empreinte: None,
            polices_introuvables: vec![],
        },
    );

    retenir(&mut o.projet, &l.cle(), Ok(&package_d_essai(&l.cle(), 120)));

    let pose = &o.projet.meta.livraison.livrables[0];
    assert_eq!(
        o.projet.meta.livraison.mesure(&gabarit).map(|m| m.pages),
        Some(120),
        "la mesure doit être celle que la composition vient de rendre"
    );
    assert_eq!(
        crate::empreinte::etat(&o.projet, pose),
        crate::empreinte::Etat::AJour,
        "empreint avant que la mesure ne soit retenue : le package naît périmé"
    );
}

/// Un échec retenu dit pourquoi, et **ne touche pas la mesure** : le pied « Vu pour » tient
/// d'une composition qui, elle, a eu lieu. L'effacer parce qu'une autre a échoué ferait
/// disparaître un dos juste devant un message d'erreur.
#[test]
fn un_echec_retient_son_message_et_laisse_la_mesure() {
    let mut o = ouvert_neuf();
    let l = o.projet.meta.livraison.livrables[0].clone();
    let gabarit = l.fabrication.cle_gabarit();
    o.projet.meta.livraison.retenir_mesure(
        &gabarit,
        Mesure {
            pages: 98,
            gouttiere: 14.0,
            blanche: false,
            empreinte: None,
            polices_introuvables: vec![],
        },
    );

    retenir(&mut o.projet, &l.cle(), Err("typst absent".into()));

    assert_eq!(
        o.projet.meta.livraison.livrables[0].generation,
        crate::projet::Generation::Echec {
            message: "typst absent".into()
        }
    );
    assert_eq!(
        o.projet.meta.livraison.mesure(&gabarit).map(|m| m.pages),
        Some(98),
        "un échec n'efface pas la mesure d'une composition qui avait réussi"
    );
}

/// Une clé que le livre ne porte plus — un livrable retiré pendant la composition — n'a
/// personne à renseigner, et ne doit pas paniquer. C'est le même parti que `retenir_mesure`,
/// qui ignore de lui-même un gabarit que plus aucun livrable ne porte.
#[test]
fn retenir_sur_une_cle_absente_ne_fait_rien() {
    let mut o = ouvert_neuf();
    retenir(
        &mut o.projet,
        "pod-inconnu-broche-creme",
        Ok(&package_d_essai("x", 100)),
    );
    assert!(o.projet.meta.livraison.livrables[0].generation.est_jamais());
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test retenir la_mesure un_echec_retient`
Attendu : ÉCHEC à la compilation — `cannot find function 'retenir' in this scope`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/commands.rs`, juste avant `pub fn packager` :

```rust
/// Ce qu'une génération laisse sur le livrable : la mesure de son gabarit d'abord, ses deux
/// empreintes ensuite.
///
/// **L'ordre n'est pas négociable** (reconnaissance 3a) : `empreinte::couverture` lit la
/// pagination que le projet retient, et prendre l'empreinte avant de retenir la mesure la
/// daterait de la composition d'avant. Le package naîtrait périmé sur sa couverture — donc
/// sur son dos — sans que rien à l'écran puisse l'expliquer.
///
/// Un livrable que la clé ne désigne plus n'a personne à renseigner : la fonction se tait,
/// comme `retenir_mesure` le fait déjà pour un gabarit que plus aucun livrable ne porte.
fn retenir(projet: &mut Projet, cle: &str, issue: Result<&package::Package, String>) {
    let Some(l) = projet
        .meta
        .livraison
        .livrables
        .iter()
        .find(|x| x.cle() == cle)
        .cloned()
    else {
        return;
    };
    let generation = match issue {
        Err(message) => crate::projet::Generation::Echec { message },
        Ok(p) => {
            // 1. La mesure, sous la clé du **gabarit** : c'est elle que l'empreinte de
            // couverture va lire deux lignes plus bas. Un gabarit que le catalogue ne porte
            // plus ne se mesure pas — le livrable paraîtra périmé, ce qui est vrai.
            if let Ok(r) = catalogue::resout(&l.fabrication) {
                projet.meta.livraison.retenir_mesure(
                    &l.fabrication.cle_gabarit(),
                    Mesure {
                        pages: p.pages,
                        gouttiere: p.gouttiere,
                        blanche: p.blanche,
                        empreinte: Some(r.empreinte()),
                        polices_introuvables: p.polices_introuvables.clone(),
                    },
                );
            }
            // 2. Les empreintes, sur le projet que la mesure vient de mettre à jour.
            crate::projet::Generation::Fait {
                interieur: crate::empreinte::interieur(projet, &l),
                couverture: crate::empreinte::couverture(projet, &l),
            }
        }
    };
    if let Some(place) = projet
        .meta
        .livraison
        .livrables
        .iter_mut()
        .find(|x| x.cle() == cle)
    {
        place.generation = generation;
    }
}
```

Puis remplacer la boucle de `packager` (`commands.rs:1656-1670`, celle qui commence par
`for (d, r) in livrables.iter().zip(&sorties)`) par :

```rust
    // Ce que la génération vient de mesurer entre dans le projet, gabarit par gabarit,
    // exactement comme la mesure de `composer` : c'est le même livre, composé par le même
    // Typst, sous la même clé de rangement. Et ce qu'elle a produit entre sur le livrable :
    // ses deux empreintes, ou le message qui dit pourquoi il n'y en a pas.
    //
    // Le consentement ne s'y oppose pas : il gouverne le déclenchement d'une composition que
    // personne n'a demandée, pas le droit de retenir celle qu'un clic vient de réclamer.
    for (d, r) in livrables.iter().zip(&sorties) {
        let issue = match (&r.package, &r.erreur) {
            (Some(p), _) => Ok(p),
            (None, e) => Err(e.clone().unwrap_or_else(|| "composition échouée.".into())),
        };
        retenir(&mut o.projet, &d.cle(), issue);
    }
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 613 passés, 0 échec.

Mutation ciblée, à voir rouge puis à défaire : intervertir les deux blocs de `retenir` —
calculer `Generation::Fait` avant l'appel à `retenir_mesure`. Attendu :
`la_mesure_est_retenue_avant_que_les_empreintes_ne_soient_prises` échoue sur
`Perime { interieur: false, couverture: true }`.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # 98 / 118 / 100, inchangé
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
git add src-tauri/src/commands.rs
git commit -m "Une génération retient sa mesure avant d'en prendre l'empreinte"
```

---

### Tâche 2 : Un intérieur déjà sur le disque se copie au lieu de se recomposer

C'est la contrepartie de générer à l'ajout (spec § 4) : trois papiers d'un même gabarit
coûtent une composition et deux copies, **qu'on les ajoute d'un coup ou un par un**. Les
cibles de la passe sont exclues du vivier (reconnaissance 4d), ce qui préserve « Tout
regénérer » à l'identique et empêche Régénérer de se réutiliser lui-même.

**Fichiers :**
- Modifier : `src-tauri/src/package.rs` (fonction neuve avant `lot`, et `lot` lui-même)

**Interfaces :**
- Consomme : `crate::empreinte::interieur`, `projet::Generation::Fait`, `Livraison::mesure`,
  `package::nom`.
- Produit : `fn interieur_du_disque(projet: &Projet, racine: &Path, gabarit: &str, exclues:
  &[&str]) -> Option<InterieurCompose>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/package.rs`, sous `#[cfg(test)] mod tests` :

```rust
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
    let gabarit = projet.meta.livraison.livrables[0].fabrication.cle_gabarit();

    let i = interieur_du_disque(&projet, racine.path(), &gabarit, &[])
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
    let gabarit = projet.meta.livraison.livrables[0].fabrication.cle_gabarit();

    assert!(
        interieur_du_disque(&projet, racine.path(), &gabarit, &[&cle]).is_none(),
        "régénérer se serait prêté son propre intérieur : il n'aurait rien recomposé"
    );
}

/// Une empreinte qui a bougé ne prête rien : c'est tout le mécanisme du lot 1. Le PDF est là,
/// il est lisible, et il ne compose plus ce livre-là.
#[test]
fn un_interieur_perime_ne_se_prete_pas() {
    let racine = tempfile::tempdir().unwrap();
    let (mut projet, _) = projet_genere(racine.path(), 266);
    let gabarit = projet.meta.livraison.livrables[0].fabrication.cle_gabarit();
    projet.remplacer_texte("## 01 - Un\n\nUn autre paragraphe.".into());
    // `remplacer_texte` oublie les mesures : on remet celle du gabarit pour prouver que c'est
    // bien l'empreinte, et non l'absence de mesure, qui refuse le prêt.
    projet.meta.livraison.retenir_mesure(
        &gabarit,
        crate::projet::Mesure {
            pages: 266,
            gouttiere: 14.0,
            blanche: false,
            empreinte: None,
            polices_introuvables: vec![],
        },
    );

    assert!(interieur_du_disque(&projet, racine.path(), &gabarit, &[]).is_none());
}

/// Un PDF effacé à la main ne se copie pas : la source seule laisserait dans le répertoire
/// livré un `.typ` qui ne correspond à rien.
#[test]
fn un_fichier_manquant_ne_se_prete_pas() {
    let racine = tempfile::tempdir().unwrap();
    let (projet, cle) = projet_genere(racine.path(), 266);
    let gabarit = projet.meta.livraison.livrables[0].fabrication.cle_gabarit();
    std::fs::remove_file(racine.path().join(&cle).join(nom(&cle, "interieur", "pdf"))).unwrap();

    assert!(interieur_du_disque(&projet, racine.path(), &gabarit, &[]).is_none());
}

/// Sans mesure, pas de pagination à prêter — et rien à inventer : `InterieurCompose` la porte,
/// et le PDF ne la dit pas.
#[test]
fn sans_mesure_rien_ne_se_prete() {
    let racine = tempfile::tempdir().unwrap();
    let (mut projet, _) = projet_genere(racine.path(), 266);
    let gabarit = projet.meta.livraison.livrables[0].fabrication.cle_gabarit();
    projet.meta.livraison.oublier_mesures();

    assert!(interieur_du_disque(&projet, racine.path(), &gabarit, &[]).is_none());
}
```

Et le test bout-en-bout que le § 8 de la spec réclame — **deux appels séparés**, ce que le test
existant `deux_livrables_du_meme_gabarit_ne_composent_l_interieur_qu_une_fois` ne prouve pas,
puisqu'il compose ses deux cibles dans la même passe :

```rust
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
    let pdf_a = racine.path().join(cle_a).join(nom(cle_a, "interieur", "pdf"));
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

    assert!(b.interieur_partage, "le second devait copier, pas recomposer");
    assert_eq!(b.pages, a.pages, "la pagination vient de la mesure retenue");
    assert_eq!(
        std::fs::read(racine.path().join(cle_b).join(nom(cle_b, "interieur", "pdf"))).unwrap(),
        b"MARQUE-DU-PREMIER",
        "l'intérieur a été recomposé au lieu d'être copié"
    );
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test prete`
Attendu : ÉCHEC à la compilation — `cannot find function 'interieur_du_disque'`.

Run : `cd src-tauri && cargo test -- --ignored un_interieur_se_reutilise`
Attendu : une fois la fonction écrite mais `lot` non branché, ÉCHEC sur
`le second devait copier, pas recomposer`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/package.rs`, juste avant `pub fn lot` :

```rust
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
    gabarit: &str,
    exclues: &[&str],
) -> Option<InterieurCompose> {
    let m = projet.meta.livraison.mesure(gabarit)?;
    projet.meta.livraison.livrables.iter().find_map(|l| {
        if l.fabrication.cle_gabarit() != gabarit {
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
```

Puis, dans `lot`, remplacer le corps du `if !prets.contains_key(&c.pr.cle)` :

```rust
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
            if !prets.contains_key(&c.pr.cle) {
                let i = match interieur_du_disque(projet, racine, &c.pr.cle, &exclues) {
                    Some(i) => i,
                    None => composer_interieur(projet, &c.pr, &c.cle, &dossier, typst)?,
                };
                prets.insert(c.pr.cle.clone(), (c.pr.clone(), i));
            }
            let (pr, interieur) = prets.get(&c.pr.cle).expect("vient d'être inséré si absent");
            debug_assert!(
                meme_gabarit(pr, &c.pr),
                "deux gabarits de même clé, providers différents"
            );
            assembler(projet, c, interieur, &dossier, typst)
        })
        .collect()
}
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 618 passés, 0 échec.
Run : `cd src-tauri && cargo test -- --ignored`
Attendu : 10 passés (les 9 d'avant et celui-ci), 0 échec. Compter plusieurs minutes.

Mutation ciblée, à voir rouge puis à défaire : supprimer les trois lignes
`if exclues.contains(&cle.as_str()) { return None; }`. Attendu :
`une_cible_de_la_passe_ne_se_prete_pas_son_propre_interieur` échoue.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # 98 / 118 / 100, inchangé
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
git add src-tauri/src/package.rs
git commit -m "Un intérieur déjà composé se copie, d'une génération à la suivante"
```

---

### Tâche 3 : Supprimer efface ce que l'application a écrit, et rien d'autre

La spec § 3 : les fichiers connus partent, le répertoire suit **s'il est vide**, un fichier
étranger survit et se nomme. Un fichier déjà parti n'est pas une erreur. Tout cela est du
disque sans Typst : c'est éprouvable tel quel, avant qu'aucune commande n'existe.

**Fichiers :**
- Modifier : `src-tauri/src/package.rs` (deux fonctions et un type, après `ecrire_table`)

**Interfaces :**
- Consomme : `package::nom`.
- Produit : `pub struct Nettoyage { absents, etrangers, dossier_retire }`,
  `pub fn fichiers_du_livrable(cle: &str, images: &[String]) -> Vec<String>`,
  `pub fn effacer_livrable(dossier: &Path, cle: &str, images: &[String]) -> Result<Nettoyage, String>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/package.rs`, sous `#[cfg(test)] mod tests` :

```rust
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
    assert!(!n.dossier_retire, "le répertoire porte encore quelque chose");
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

    assert_eq!(n.absents.len(), 6, "les six fichiers connus manquaient : {n:?}");
    assert!(n.etrangers.is_empty());
    assert!(!n.dossier_retire, "il n'y avait pas de répertoire à retirer");
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test fichiers_du_livrable etranger repertoire`
Attendu : ÉCHEC à la compilation — `cannot find function 'fichiers_du_livrable'`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/package.rs`, après `ecrire_table` :

```rust
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
pub fn effacer_livrable(
    dossier: &Path,
    cle: &str,
    images: &[String],
) -> Result<Nettoyage, String> {
    let mut n = Nettoyage::default();
    for f in fichiers_du_livrable(cle, images) {
        match std::fs::remove_file(dossier.join(&f)) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => n.absents.push(f),
            Err(e) => return Err(format!("{f} ne s'efface pas : {e}")),
        }
    }
    // Ce qui reste porte le répertoire : on le lit avant de tenter de le retirer, pour pouvoir
    // le nommer. Un répertoire absent ne se lit pas, et n'a rien laissé.
    let Ok(reste) = std::fs::read_dir(dossier) else {
        return Ok(n);
    };
    for e in reste.flatten() {
        n.etrangers.push(e.file_name().to_string_lossy().into_owned());
    }
    n.etrangers.sort();
    if n.etrangers.is_empty() {
        std::fs::remove_dir(dossier).map_err(|e| {
            format!("le répertoire ne se retire pas ({}) : {e}", dossier.display())
        })?;
        n.dossier_retire = true;
    }
    Ok(n)
}
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 622 passés, 0 échec.

Mutation ciblée, à voir rouge puis à défaire : remplacer le corps d'`effacer_livrable` par
`std::fs::remove_dir_all(dossier)`. Attendu : `un_fichier_etranger_survit_et_se_nomme` échoue —
c'est exactement la décision de cadrage que ce test protège.

- [ ] **Étape 5 : commit**

```bash
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
git add src-tauri/src/package.rs
git commit -m "Supprimer un livrable efface ce que l'application a écrit, et le dit"
```

---

### Tâche 4 : Générer et régénérer, par le même chemin que Tout regénérer

Le corps de `packager` sort en fonction libre, appelée avec une cible ou avec toutes. C'est le
§ 4 de la spec — « Générer un livrable devient alors `lot` avec une seule cible ; "Tout
regénérer", `lot` avec toutes. Un seul chemin de composition, un seul jeu de garanties. »

Les commandes prennent un `State` qu'aucun test ne fabrique : leur corps descend donc dans des
fonctions qui prennent `&mut Ouvert`, que `ouvert_neuf()` fabrique déjà. C'est l'extension
d'une manière que le dépôt a prise cinq fois — `refuse_doublon`, `reglage_refuse`,
`meme_gabarit`, `dossiers_d_envoi`, `verifie_pagination` : la règle vit hors de la commande
pour être éprouvée.

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (`packager` allégée, trois fonctions neuves, deux
  commandes neuves)
- Modifier : `src-tauri/src/lib.rs` (deux lignes dans l'`invoke_handler`)

**Interfaces :**
- Consomme : `retenir` (tâche 1), `package::lot` (tâche 2), `cible`, `sorties_racine`,
  `refuse_doublon`, `nom_finition`.
- Produit :
  - `fn composer_lot(o: &mut Ouvert, cles: &[String], typst: &Typst) -> Result<Generation, String>`
  - `fn generer(o: &mut Ouvert, fabrication: catalogue::Fabrication, typst: &Typst) -> Result<Generation, String>`
  - `fn regenerer(o: &mut Ouvert, cle: &str, typst: &Typst) -> Result<Generation, String>`
  - `#[tauri::command] pub fn livrable_generer(fabrication: catalogue::Fabrication, atelier: State<Atelier>) -> Result<Generation, String>`
  - `#[tauri::command] pub fn livrable_regenerer(cle: String, atelier: State<Atelier>) -> Result<Generation, String>`

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/commands.rs`, sous `#[cfg(test)] mod tests` :

```rust
/// Un atelier ouvert sur un projet enregistré : `sorties_racine` réclame un chemin, et sans
/// lui aucune composition ne peut être tentée.
fn ouvert_enregistre(dir: &tempfile::TempDir) -> Ouvert {
    Ouvert {
        chemin: Some(dir.path().join("livre.ozalid")),
        ..ouvert_neuf()
    }
}

/// Une fabrication du catalogue réel, différente de celle du livrable d'office : le deuxième
/// papier du premier POD. C'est un livrable que `catalogue::resout` accepte, et qui n'est pas
/// le doublon de celui que `Projet::nouveau` a posé.
fn fabrication_seconde(o: &Ouvert) -> catalogue::Fabrication {
    let d = o.projet.meta.livraison.livrables[0].fabrication.clone();
    let pod = catalogue::pod(&d.pod).expect("le POD du livrable d'office");
    let autre = pod
        .papiers
        .iter()
        .find(|p| p.cle != d.papier)
        .expect("le premier POD du catalogue porte au moins deux papiers");
    catalogue::Fabrication {
        papier: autre.cle.clone(),
        ..d
    }
}

/// **La racine se vérifie avant que le livrable ne soit posé.** Un projet jamais enregistré
/// n'a pas de répertoire de sorties ; poser le livrable puis buter dessus laisserait dans le
/// livre un livrable *jamais généré* que personne n'a demandé, sous un message qui parle
/// d'autre chose. C'est l'ordre que `livrable_regler` prend déjà pour son candidat.
#[test]
fn generer_sans_projet_enregistre_ne_pose_rien() {
    let mut o = ouvert_neuf(); // sans chemin
    let avant = o.projet.meta.livraison.livrables.len();
    let f = fabrication_seconde(&o);

    let e = generer(&mut o, f, &Typst::new("typst-absent")).unwrap_err();

    assert!(e.contains("enregistrer"), "{e}");
    assert_eq!(o.projet.meta.livraison.livrables.len(), avant);
}

/// **Un échec de composition crée quand même le livrable** (spec § 3) : il paraît en erreur,
/// avec son message, et son bouton Régénérer. Sans quoi la seule issue serait de tout
/// ressaisir — cinq listes déroulantes pour un sidecar absent.
#[test]
fn un_echec_de_composition_laisse_le_livrable_en_erreur() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let f = fabrication_seconde(&o);
    let cle = f.cle();

    let g = generer(&mut o, f, &Typst::new("typst-absent")).expect("le livrable est posé");

    assert_eq!(g.packages.len(), 1, "une cible, un résultat");
    assert!(g.packages[0].erreur.is_some());
    let pose = o
        .projet
        .meta
        .livraison
        .livrables
        .iter()
        .find(|l| l.cle() == cle)
        .expect("le livrable reste dans le livre");
    assert!(
        matches!(pose.generation, crate::projet::Generation::Echec { .. }),
        "l'échec doit être retenu sur le livrable : {:?}",
        pose.generation
    );
}

/// Le doublon se refuse comme à l'ajout, et **avant** de composer : composer pour découvrir
/// ensuite qu'on refuse coûterait des secondes pour rien, et écrirait des fichiers dans le
/// répertoire d'un livrable qui existe déjà.
#[test]
fn generer_refuse_un_doublon_sans_composer() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let f = o.projet.meta.livraison.livrables[0].fabrication.clone();

    let e = generer(&mut o, f, &Typst::new("typst-absent")).unwrap_err();

    assert!(e.contains("déjà un livrable"), "{e}");
    assert_eq!(o.projet.meta.livraison.livrables.len(), 1);
    assert!(!dir.path().join("livre").exists(), "rien n'a été écrit");
}

/// Régénérer ne touche à aucun axe : le livrable est le même après qu'avant, seul son état
/// change. Et une clé que le livre ne porte pas se refuse en le disant.
#[test]
fn regenerer_ne_touche_pas_aux_axes_et_refuse_une_cle_inconnue() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let avant = o.projet.meta.livraison.livrables[0].clone();

    let g = regenerer(&mut o, &avant.cle(), &Typst::new("typst-absent"))
        .expect("la commande rend son compte rendu, l'échec est dans le résultat");
    assert_eq!(g.packages.len(), 1);
    assert_eq!(
        o.projet.meta.livraison.livrables[0].fabrication,
        avant.fabrication
    );

    let e =
        regenerer(&mut o, "pod-inconnu-broche-creme", &Typst::new("typst-absent")).unwrap_err();
    assert!(e.contains("n'est pas un livrable"), "{e}");
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test generer regenerer`
Attendu : ÉCHEC à la compilation — `cannot find function 'generer'`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/commands.rs`, poser les trois fonctions avant `packager` :

```rust
/// Compose les livrables que ces clés désignent, chacun dans son répertoire, et retient ce
/// que la composition a laissé.
///
/// **Le seul chemin de composition** (spec § 4) : générer un livrable, c'est cette fonction
/// avec une clé ; « Tout regénérer », c'est elle avec toutes. Un seul jeu de garanties — la
/// résolution d'abord, la mémoïsation de l'intérieur par gabarit ensuite, l'ordre
/// mesure-puis-empreinte à la fin.
fn composer_lot(o: &mut Ouvert, cles: &[String], typst: &Typst) -> Result<Generation, String> {
    let livrables: Vec<Livrable> = o
        .projet
        .meta
        .livraison
        .livrables
        .iter()
        .filter(|l| cles.contains(&l.cle()))
        .cloned()
        .collect();
    if livrables.is_empty() {
        return Err("aucun livrable : en déclarer un.".into());
    }

    // Résolution d'abord : un axe ou un papier inconnu se fige en `Resultat` d'erreur ici,
    // sans passer par le lot. Le reste devient une `Cible`, dans l'ordre des livrables —
    // c'est cet ordre que la fin de la fonction restitue.
    let mut etapes: Vec<Result<package::Cible, Resultat>> = Vec::with_capacity(livrables.len());
    for d in &livrables {
        etapes.push(match catalogue::resout(&d.fabrication) {
            Ok(r) => Ok(cible(r.provider(), r.papier.clone(), d)),
            // Le POD est le seul axe qui puisse encore se nommer quand la résolution échoue
            // sur un autre : afficher la clé à quatre segments en gros titre serait un recul
            // devant « BoD ».
            Err(e) => {
                let pod = catalogue::pod(&d.fabrication.pod);
                Err(Resultat {
                    cle: d.cle(),
                    libelle: pod.map(|p| p.nom.clone()).unwrap_or_else(|| d.cle()),
                    finition: nom_finition(d, pod),
                    package: None,
                    vignette: None,
                    erreur: Some(e),
                })
            }
        });
    }

    // `?` fait échouer la commande entière, sans `Resultat` par livrable : à la différence
    // d'un POD ou d'un papier inconnu, une racine de sorties inutilisable (projet non
    // enregistré) ne concerne aucun livrable en particulier, et rien ne peut être tenté avant
    // qu'elle existe.
    let racine = sorties_racine(o)?;
    let cibles: Vec<package::Cible> = etapes
        .iter()
        .filter_map(|e| e.as_ref().ok().cloned())
        .collect();
    let mut paquets = package::lot(&o.projet, &cibles, &racine, typst).into_iter();

    // `zip` sur les livrables : `etapes` a été poussée dans leur ordre, et la finition ne
    // voyage pas dans la `Cible` — elle ne fabrique rien, aucun octet du PDF ni aucun nom de
    // fichier n'en dépend. Elle se commande, et le récapitulatif est le seul endroit où elle
    // peut être lue.
    let sorties: Vec<Resultat> = etapes
        .into_iter()
        .zip(&livrables)
        .map(|(etape, d)| match etape {
            Err(r) => r,
            Ok(cible) => {
                let finition = nom_finition(d, catalogue::pod(&d.fabrication.pod));
                match paquets.next().expect("un résultat par cible envoyée à lot") {
                    Ok(p) => Resultat {
                        cle: cible.cle,
                        libelle: cible.pr.libelle,
                        finition,
                        // La vignette manquante ne perd pas le package : les PDF sont écrits,
                        // et c'est eux que l'imprimeur reçoit.
                        vignette: donnee_png(Path::new(&p.vignette)).ok(),
                        package: Some(p),
                        erreur: None,
                    },
                    Err(e) => Resultat {
                        cle: cible.cle,
                        libelle: cible.pr.libelle,
                        finition,
                        package: None,
                        vignette: None,
                        erreur: Some(e),
                    },
                }
            }
        })
        .collect();

    // Ce que la génération vient de mesurer entre dans le projet, gabarit par gabarit, et ce
    // qu'elle a produit entre sur le livrable : ses deux empreintes, ou le message qui dit
    // pourquoi il n'y en a pas. L'ordre des deux est dans `retenir`.
    for (d, r) in livrables.iter().zip(&sorties) {
        let issue = match (&r.package, &r.erreur) {
            (Some(p), _) => Ok(p),
            (None, e) => Err(e.clone().unwrap_or_else(|| "composition échouée.".into())),
        };
        retenir(&mut o.projet, &d.cle(), issue);
    }

    Ok(Generation {
        projet: vue_modifiee(o)?,
        packages: sorties,
    })
}

/// Pose le livrable, puis compose. Cet ordre est ce qui donne une place à l'échec (spec § 3) :
/// une composition ratée laisse un livrable en erreur, pas cinq listes à ressaisir.
///
/// La racine des sorties se vérifie **avant** la pose : elle ne concerne aucun livrable en
/// particulier, et poser pour buter dessus laisserait dans le livre un livrable que personne
/// n'a demandé.
fn generer(
    o: &mut Ouvert,
    fabrication: catalogue::Fabrication,
    typst: &Typst,
) -> Result<Generation, String> {
    let r = catalogue::resout(&fabrication)?;
    sorties_racine(o)?;
    let cle = fabrication.cle();
    if refuse_doublon(&o.projet.meta.livraison.livrables, &cle) {
        return Err(format!(
            "{} en {} est déjà un livrable de ce livre — la finition seule n'en fait \
             pas un autre : le fichier produit serait le même.",
            r.pod.nom, r.papier.nom
        ));
    }
    o.projet
        .meta
        .livraison
        .livrables
        .push(Livrable::pour(fabrication));
    composer_lot(o, std::slice::from_ref(&cle), typst)
}

/// Recompose un livrable sans toucher à ses axes.
fn regenerer(o: &mut Ouvert, cle: &str, typst: &Typst) -> Result<Generation, String> {
    if !o
        .projet
        .meta
        .livraison
        .livrables
        .iter()
        .any(|l| l.cle() == cle)
    {
        return Err(format!("{cle} n'est pas un livrable de ce livre."));
    }
    composer_lot(o, &[cle.to_string()], typst)
}
```

`packager` devient :

```rust
/// Génère le package de chaque livrable du livre, chacun dans son répertoire.
///
/// Une seule maquette, N livrables, aucun réglage retouché entre eux : chacun compose son
/// propre intérieur, donc sa propre pagination, donc son propre dos. C'est « Tout regénérer »
/// de l'étape Livraison — la même fonction que les verbes unitaires, avec toutes les clés.
#[tauri::command]
pub fn packager(atelier: State<Atelier>) -> Result<Generation, String> {
    let typst = typst()?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let cles: Vec<String> = o
        .projet
        .meta
        .livraison
        .livrables
        .iter()
        .map(|l| l.cle())
        .collect();
    composer_lot(o, &cles, &typst)
}
```

Et les deux commandes neuves, à la suite de `livrable_viser` :

```rust
/// Pose un livrable et compose son package dans la foulée : un livrable naît généré.
#[tauri::command]
pub fn livrable_generer(
    fabrication: catalogue::Fabrication,
    atelier: State<Atelier>,
) -> Result<Generation, String> {
    let typst = typst()?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    generer(o, fabrication, &typst)
}

/// Recompose un livrable sans toucher à ses axes.
#[tauri::command]
pub fn livrable_regenerer(cle: String, atelier: State<Atelier>) -> Result<Generation, String> {
    let typst = typst()?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    regenerer(o, &cle, &typst)
}
```

Dans `src-tauri/src/lib.rs`, après `commands::livrable_viser,` :

```rust
            commands::livrable_generer,
            commands::livrable_regenerer,
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 626 passés, 0 échec.

Mutation ciblée, à voir rouge puis à défaire : dans `generer`, déplacer `sorties_racine(o)?`
après le `push`. Attendu : `generer_sans_projet_enregistre_ne_pose_rien` échoue sur le compte
de livrables.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # 98 / 118 / 100, inchangé
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd .. && node --test tests/*.test.js            # le front n'a pas bougé : 305/0
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Un livrable naît généré, et se régénère par le même chemin"
```

---

### Tâche 5 : Remplacer compose avant d'effacer

Si la nouvelle composition échouait après qu'on a vidé l'ancien répertoire, on aurait échangé
un package qui marchait contre un qui ne marche pas (spec § 3). L'ordre est : composer, puis
effacer l'ancien, et seulement en cas de succès.

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (une fonction et une commande)
- Modifier : `src-tauri/src/lib.rs` (une ligne)

**Interfaces :**
- Consomme : `composer_lot` (tâche 4), `package::effacer_livrable` (tâche 3), `refuse_doublon`,
  `sorties_racine`.
- Produit : `fn remplacer(o: &mut Ouvert, cle: &str, livrable: Livrable, typst: &Typst) ->
  Result<Generation, String>`, `#[tauri::command] pub fn livrable_remplacer(cle: String,
  livrable: Livrable, atelier: State<Atelier>) -> Result<Generation, String>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/commands.rs`, sous `#[cfg(test)] mod tests` :

```rust
/// **L'ancien package survit à une composition ratée.** C'est tout le sens de l'ordre
/// composer-puis-effacer : sans lui, on aurait échangé un package qui marchait contre un qui
/// ne marche pas, et sans recours.
#[test]
fn un_remplacement_rate_laisse_l_ancien_package_intact() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let ancien = o.projet.meta.livraison.livrables[0].clone();
    let racine = sorties_racine(&o).unwrap();
    let dossier = racine.join(ancien.cle());
    std::fs::create_dir_all(&dossier).unwrap();
    let pdf = dossier.join(package::nom(&ancien.cle(), "interieur", "pdf"));
    std::fs::write(&pdf, b"%PDF-ancien").unwrap();

    let neuf = Livrable::pour(fabrication_seconde(&o));
    let g = remplacer(&mut o, &ancien.cle(), neuf, &Typst::new("typst-absent"))
        .expect("le remplacement rend son compte rendu");

    assert!(
        g.packages[0].erreur.is_some(),
        "la composition devait échouer, Typst est absent"
    );
    assert_eq!(
        std::fs::read(&pdf).unwrap(),
        b"%PDF-ancien",
        "l'ancien package a été effacé avant que le neuf ne soit acquis"
    );
    assert_eq!(
        o.projet.meta.livraison.livrables[0].cle(),
        ancien.cle(),
        "le livre doit rester sur l'ancien livrable : lui seul a un package, et rien d'autre \
         ne pourrait plus le nommer"
    );
}

/// Un livrable ne se refuse pas lui-même au titre du doublon : c'est le même, on le règle.
/// Sans cette exception, corriger un relevé serait impossible.
#[test]
fn remplacer_ne_se_refuse_pas_lui_meme() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let ancien = o.projet.meta.livraison.livrables[0].clone();
    let mut neuf = ancien.clone();
    neuf.dos_mm = Some(18.4);

    let r = remplacer(&mut o, &ancien.cle(), neuf, &Typst::new("typst-absent"));

    // La composition échoue — Typst est absent —, mais pas le remplacement : le refus de
    // doublon aurait, lui, échoué avant toute composition et avec un autre message.
    assert!(
        r.is_ok(),
        "le livrable s'est refusé à lui-même : {}",
        r.unwrap_err()
    );
    assert_eq!(o.projet.meta.livraison.livrables[0].dos_mm, Some(18.4));
}

/// Une finition que le POD ne porte pas se refuse : elle nomme une option de commande, et une
/// option inventée ne se commande nulle part. C'est le seul membre de `reglage_refuse` qui
/// survit au remplacement — le POD et le format, eux, se changent désormais, puisque Remplacer
/// recompose.
#[test]
fn remplacer_refuse_une_finition_etrangere_au_pod() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let ancien = o.projet.meta.livraison.livrables[0].clone();
    let mut neuf = ancien.clone();
    neuf.finition = Some("dorure-a-chaud-inventee".into());

    let e = remplacer(&mut o, &ancien.cle(), neuf, &Typst::new("typst-absent")).unwrap_err();

    assert!(e.contains("finition inconnue"), "{e}");
    assert!(o.projet.meta.livraison.livrables[0].finition.is_none());
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test remplacer remplacement`
Attendu : ÉCHEC à la compilation — `cannot find function 'remplacer'`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/commands.rs` :

```rust
/// Remplace un livrable par un autre, à sa place, et recompose.
///
/// **Composer avant d'effacer** (spec § 3) : si la nouvelle composition échouait après qu'on a
/// vidé l'ancien répertoire, on aurait échangé un package qui marchait contre un qui ne marche
/// pas. L'ancien répertoire n'est donc effacé qu'après une composition réussie, et seulement
/// si la clé a bougé — à clé égale, c'est le même répertoire, et la composition vient d'y
/// réécrire.
///
/// Le POD et le format se changent ici, quand `livrable_regler` les refusait : ce refus tenait
/// à ce que régler ne recomposait pas. Ce qui en survit est la finition, qui doit exister chez
/// l'imprimeur.
///
/// Le rang suit la décision du 29/08 : conservé quand le POD ne change pas, poussé en queue
/// quand il change — le livrable entre alors dans son nouveau groupe comme s'il venait d'y
/// être ajouté.
///
/// Une composition ratée rend la pose **quand la clé a bougé**, et la garde à clé égale : voir
/// le corps, où les deux cas sont dits.
fn remplacer(
    o: &mut Ouvert,
    cle: &str,
    livrable: Livrable,
    typst: &Typst,
) -> Result<Generation, String> {
    // Le candidat est résolu **avant** d'être posé : un axe ou un papier inconnu doit laisser
    // le livrable tel qu'il était, et non l'abandonner à moitié réglé.
    let r = catalogue::resout(&livrable.fabrication)?;
    if let Some(f) = &livrable.finition {
        if !r.pod.finitions.iter().any(|x| &x.cle == f) {
            return Err(format!("finition inconnue chez {} : {f}.", r.pod.nom));
        }
    }
    let racine = sorties_racine(o)?;
    let l = &mut o.projet.meta.livraison;
    let neuve = livrable.cle();
    let rang = l
        .livrables
        .iter()
        .position(|x| x.cle() == cle)
        .ok_or_else(|| format!("{cle} n'est pas un livrable de ce livre."))?;
    // Le livrable édité ne se compte pas comme doublon de lui-même (spec § 3).
    if neuve != cle && refuse_doublon(&l.livrables, &neuve) {
        return Err(format!("{neuve} est déjà un livrable de ce livre."));
    }
    let ancien = l.livrables[rang].clone();
    let change_de_pod = ancien.fabrication.pod != livrable.fabrication.pod;
    if change_de_pod {
        l.livrables.remove(rang);
        l.livrables.push(livrable);
    } else {
        l.livrables[rang] = livrable;
    }
    if l.courant == cle {
        l.courant = neuve.clone();
    }

    let g = composer_lot(o, std::slice::from_ref(&neuve), typst)?;
    if g.packages.iter().any(|p| p.erreur.is_some()) {
        // Un échec ne laisse pas effacer l'ancien : le package neuf n'existe pas.
        //
        // Et **la pose est rendue quand la clé a bougé** : la composition ratée écrivait dans
        // un autre répertoire, celui de l'ancien est intact, et son état reste vrai. Le
        // garder posé mettrait hors de portée de l'application un package qui marche — plus
        // rien ne le nommerait, donc plus rien ne pourrait l'effacer. Le projet ne touche pas
        // le disque avant l'enregistrement : la restauration est exacte.
        //
        // À clé égale, au contraire, la composition a écrit dans le répertoire de ce
        // livrable-là : le neuf reste posé, en erreur, parce que c'est la vérité de ses
        // fichiers — et ses axes sont ceux de l'ancien, à un relevé près.
        if neuve != cle {
            let l = &mut o.projet.meta.livraison;
            if change_de_pod {
                l.livrables.pop();
                l.livrables.insert(rang, ancien.clone());
            } else {
                l.livrables[rang] = ancien.clone();
            }
            if l.courant == neuve {
                l.courant = cle.to_string();
            }
            return Ok(Generation {
                projet: vue_modifiee(o)?,
                packages: g.packages,
            });
        }
        return Ok(g);
    }
    if neuve != cle {
        let images: Vec<String> = o.projet.images.keys().cloned().collect();
        package::effacer_livrable(&racine.join(cle), cle, &images)?;
    }
    Ok(g)
}
```

Et la commande, à la suite de `livrable_regenerer` :

```rust
/// Remplace un livrable par celui que le formulaire porte, et recompose.
#[tauri::command]
pub fn livrable_remplacer(
    cle: String,
    livrable: Livrable,
    atelier: State<Atelier>,
) -> Result<Generation, String> {
    let typst = typst()?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    remplacer(o, &cle, livrable, &typst)
}
```

Dans `src-tauri/src/lib.rs`, après `commands::livrable_regenerer,` :

```rust
            commands::livrable_remplacer,
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 629 passés, 0 échec.

Mutation ciblée, à voir rouge puis à défaire : déplacer l'appel à `effacer_livrable` **avant**
`composer_lot`. Attendu : `un_remplacement_rate_laisse_l_ancien_package_intact` échoue sur la
lecture du PDF effacé.

**Réserve à porter au compte rendu du lot, non traitée ici :** un remplacement qui échoue après
avoir changé de clé rend la pose, mais laisse dans le **nouveau** répertoire les fichiers
partiels que la composition avait écrits avant de buter. Aucun livrable ne les porte. Les
effacer au rattrapage serait une décision de produit que la spec ne prend pas ; le lot 3 doit
seulement savoir que ce répertoire-là peut exister.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # 98 / 118 / 100, inchangé
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Remplacer un livrable compose avant d'effacer l'ancien"
```

---

### Tâche 6 : Supprimer, en nommant ce qui reste

**Fichiers :**
- Modifier : `src-tauri/src/commands.rs` (un type, une fonction, une commande)
- Modifier : `src-tauri/src/lib.rs` (une ligne)

**Interfaces :**
- Consomme : `package::effacer_livrable` et `package::Nettoyage` (tâche 3), `sorties_racine`.
- Produit : `pub struct Suppression { projet: ProjetVue, nettoyage: package::Nettoyage }`,
  `fn supprimer(o: &mut Ouvert, cle: &str) -> Result<Suppression, String>`,
  `#[tauri::command] pub fn livrable_supprimer(cle: String, atelier: State<Atelier>) ->
  Result<Suppression, String>`.

- [ ] **Étape 1 : écrire les tests qui échouent**

Dans `src-tauri/src/commands.rs`, sous `#[cfg(test)] mod tests` :

```rust
/// Un livre garde au moins un livrable : c'est lui qui donne le format sous lequel on regarde
/// la couverture. Le refus tombe **avant** l'effacement — sinon on laisserait un livrable sans
/// package, ce qui est pire que les deux états qu'on voulait éviter.
#[test]
fn supprimer_le_dernier_livrable_se_refuse_sans_rien_effacer() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let seul = o.projet.meta.livraison.livrables[0].clone();
    let dossier = sorties_racine(&o).unwrap().join(seul.cle());
    std::fs::create_dir_all(&dossier).unwrap();
    let pdf = dossier.join(package::nom(&seul.cle(), "interieur", "pdf"));
    std::fs::write(&pdf, b"%PDF").unwrap();

    let e = supprimer(&mut o, &seul.cle()).unwrap_err();

    assert!(e.contains("au moins un livrable"), "{e}");
    assert!(pdf.is_file(), "le refus ne doit rien effacer");
}

/// Supprimer efface les fichiers, retire le livrable, et rend le pointeur à quelqu'un : celui
/// qu'on visait s'en va, `courant` retombe sur le premier plutôt que de désigner un absent
/// jusqu'au prochain geste.
#[test]
fn supprimer_efface_les_fichiers_retire_le_livrable_et_rend_le_pointeur() {
    let dir = tempfile::tempdir().unwrap();
    let mut o = ouvert_enregistre(&dir);
    let second = Livrable::pour(fabrication_seconde(&o));
    o.projet.meta.livraison.livrables.push(second.clone());
    o.projet.meta.livraison.courant = second.cle();
    let dossier = sorties_racine(&o).unwrap().join(second.cle());
    std::fs::create_dir_all(&dossier).unwrap();
    for f in package::fichiers_du_livrable(&second.cle(), &[]) {
        std::fs::write(dossier.join(f), b"x").unwrap();
    }

    let s = supprimer(&mut o, &second.cle()).unwrap();

    assert!(s.nettoyage.dossier_retire, "{:?}", s.nettoyage);
    assert!(!dossier.exists());
    assert_eq!(o.projet.meta.livraison.livrables.len(), 1);
    assert_eq!(
        o.projet.meta.livraison.courant,
        o.projet.meta.livraison.livrables[0].cle(),
        "le pointeur ne doit pas désigner un absent"
    );
}
```

- [ ] **Étape 2 : voir les tests échouer**

Run : `cd src-tauri && cargo test supprimer`
Attendu : ÉCHEC à la compilation — `cannot find function 'supprimer'`.

- [ ] **Étape 3 : écrire l'implémentation**

Dans `src-tauri/src/commands.rs` :

```rust
/// Ce que rend une suppression : le projet tel qu'elle l'a laissé, et ce que le répertoire a
/// gardé.
#[derive(Serialize)]
pub struct Suppression {
    pub projet: ProjetVue,
    pub nettoyage: package::Nettoyage,
}

/// Efface les fichiers du livrable, puis le retire du livre.
///
/// Cet ordre-là (spec § 3) : un effacement qui refuse — un fichier verrouillé — laisse le
/// livrable en place, avec ses fichiers, plutôt qu'un livre qui ne parle plus d'un répertoire
/// qui existe encore.
///
/// Le dernier livrable ne se supprime pas, comme il ne se retirait pas : c'est lui qui donne
/// le format sous lequel on regarde la couverture.
fn supprimer(o: &mut Ouvert, cle: &str) -> Result<Suppression, String> {
    let l = &o.projet.meta.livraison;
    if l.livrables.len() < 2 {
        return Err(
            "un livre garde au moins un livrable : c'est lui qui donne le format \
             sous lequel on regarde la couverture."
                .into(),
        );
    }
    if !l.livrables.iter().any(|d| d.cle() == cle) {
        return Err(format!("{cle} n'est pas un livrable de ce livre."));
    }
    let racine = sorties_racine(o)?;
    let images: Vec<String> = o.projet.images.keys().cloned().collect();
    let nettoyage = package::effacer_livrable(&racine.join(cle), cle, &images)?;

    let l = &mut o.projet.meta.livraison;
    l.livrables.retain(|d| d.cle() != cle);
    // Supprimer celui qu'on visait laisse le pointeur en l'air : il retombe sur le premier,
    // plutôt que de désigner un absent jusqu'au prochain geste.
    if l.courant().is_none() {
        l.courant = l.livrables[0].cle();
    }
    Ok(Suppression {
        projet: vue_modifiee(o)?,
        nettoyage,
    })
}
```

Et la commande, à la suite de `livrable_remplacer` :

```rust
/// Retire un livrable du livre et efface ce que l'application avait écrit pour lui.
#[tauri::command]
pub fn livrable_supprimer(cle: String, atelier: State<Atelier>) -> Result<Suppression, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    supprimer(o, &cle)
}
```

Dans `src-tauri/src/lib.rs`, après `commands::livrable_remplacer,` :

```rust
            commands::livrable_supprimer,
```

- [ ] **Étape 4 : voir les tests passer**

Run : `cd src-tauri && cargo test`
Attendu : 631 passés, 0 échec.
Run : `cd src-tauri && cargo test -- --ignored`
Attendu : 10 passés, 0 échec.

Mutation ciblée, à voir rouge puis à défaire : déplacer le contrôle `l.livrables.len() < 2`
après l'appel à `effacer_livrable`. Attendu :
`supprimer_le_dernier_livrable_se_refuse_sans_rien_effacer` échoue sur `pdf.is_file()`.

- [ ] **Étape 5 : le témoin, puis commit**

```bash
cd src-tauri && cargo run --example temoin      # 98 / 118 / 100, inchangé
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
cd .. && node --test tests/*.test.js
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "Supprimer un livrable retire ses fichiers, et dit ce qui restait"
```

---

## À l'œil, avant de clore le lot

Le front n'est pas touché : l'écran de la Livraison doit se comporter **exactement** comme
avant. À vérifier sur le projet réel, l'application relancée (`touch src-tauri/src/lib.rs`
avant `cargo build` si seul `src/` avait bougé) :

1. « Générer les packages » compose comme avant, et le compte rendu est identique.
2. Le pied « Vu pour » montre le même dos qu'avant le lot.
3. Enregistrer le projet, puis ouvrir le `.ozalid` dans un éditeur : chaque livrable généré
   porte désormais une sous-table `[livraison.livrables.generation]` avec `etat = "fait"` et
   ses deux empreintes. Un `.ozalid` d'avant, rouvert sans regénérer, n'en porte aucune.
4. Générer, puis regénérer sans rien changer : le second passage doit être **aussi lent** que
   le premier — toutes les cibles sont dans la passe, aucune amorce. C'est la contrepartie
   assumée du verdict 4d.

## Ce que ce lot ne fait pas

- **L'écran.** Le formulaire à deux verbes, le groupement par imprimeur, les quatre boutons, la
  disparition de la zone intermédiaire et le README sont le lot 3.
- **`Etat` dans la vue.** `livraison_vue` ne change pas de signature ici (décision 4).
- **Les trois anciennes commandes.** `livrable_ajouter`, `livrable_regler` et
  `livrable_retirer` restent, puisque l'écran actuel les appelle encore.
- **La garde en deux temps de Supprimer.** Le premier clic qui arme et le second qui retire
  sont un dispositif d'écran ; la commande, elle, supprime quand on l'appelle.
