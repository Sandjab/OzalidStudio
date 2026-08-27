'use strict';

/**
 * L'étape Livraison : les livrables et leurs packages. Les envois ont leur
 * étape et leur fichier, `envois.js`.
 *
 * Même partage que `couverture.js` : ce fichier ne pose aucun écouteur et ne lit pas
 * le DOM au chargement. Il définit, `app.js` branche — c'est ce qui permet aux deux de
 * vivre dans le même contexte global sans dépendre de l'ordre de chargement.
 */

/**
 * Les fichiers de catalogue du poste que le démarrage n'a pas pu lire.
 *
 * Muet sur un poste qui n'en dépose aucun — le cas de presque tous. Quand il parle, il
 * nomme le fichier et la raison : un POD absent de la liste sans explication se lirait
 * comme un POD qui n'existe pas, et la faute se chercherait dans le fichier plutôt que
 * dans sa syntaxe.
 */
async function afficherRefusCatalogue() {
  const refus = await invoke('catalogue_refus');
  const box = $('refusCatalogue');
  box.hidden = refus.length === 0;
  box.replaceChildren();
  for (const r of refus) {
    // Le nom d'abord, jamais tronqué, son répertoire ensuite et le chemin entier au
    // survol : ce découpage est celui des projets récents, et pour la même raison —
    // ces fichiers sortent tous du même `pods/`, et coupés par la fin ils se liraient
    // tous pareil. Le montage, lui, est celui de l'entête : une seule ligne où le
    // répertoire prend la place qui reste, là où les récents empilent. Les deux
    // séparateurs, comme les récents : l'application est aussi empaquetée pour Windows.
    const coupe = Math.max(r.fichier.lastIndexOf('/'), r.fichier.lastIndexOf('\\'));
    const fichier = h('span', undefined, 'fichier');
    fichier.title = r.fichier;
    fichier.append(
      h('span', `Catalogue non chargé : ${r.fichier.slice(coupe + 1)}`),
      h('span', r.fichier.slice(0, Math.max(coupe, 0)), 'chemin')
    );
    // La raison sur sa propre ligne : c'est la seule partie du message qui dit quoi
    // corriger, et elle doit se lire jusqu'au bout même quand elle fait trois phrases.
    const ligne = h('p', undefined, 'note alerte');
    ligne.append(fichier, h('span', r.raison));
    box.append(ligne);
  }
  if (refus.length) {
    // Les deux choses que l'utilisateur ne peut pas déduire de l'écran : que la liste
    // ci-dessous est complète de ce qui a pu être chargé, et qu'un fichier corrigé ne
    // sera relu qu'au prochain démarrage.
    box.append(h('p', 'Les POD fournis, eux, sont là. Corriger '
      + `${refus.length > 1 ? 'ces fichiers' : 'ce fichier'}, puis relancer : ceux du `
      + 'poste ne sont lus qu\'au démarrage.', 'note'));
  }
}

/**
 * Ce que l'ouverture a retiré de la liste.
 *
 * Même traitement que les fichiers de catalogue refusés, et à la suite : on ne peut pas
 * rétablir un livrable dont le catalogue ne porte plus l'axe, mais on peut dire lequel.
 * Sans ce mot, un livrable réglé disparaît entre deux ouvertures et le livre paraît
 * s'être défait tout seul.
 *
 * Muet quand rien n'a été retiré — presque toujours : une boîte qui s'afficherait vide
 * à chaque ouverture serait pire que le silence qu'elle corrige.
 */
function majElagues(vue) {
  const partis = vue.elagues ?? [];
  const box = $('livrablesElagues');
  box.hidden = partis.length === 0;
  // La cause dans la phrase, comme les refus la donnent : c'est elle qui dit où aller
  // corriger — un fichier de `pods/` du poste, ou un livre venu d'une machine mieux
  // pourvue. Sans elle, la disparition se lirait comme une perte du fichier lui-même.
  //
  // La clé brute, et non un libellé, et ce n'est pas un niveau de langage à améliorer :
  // c'est le seul qui reste. Ces clés portent quatre axes quand `libelleProvider` en
  // attend trois, et lui il retombe **de toute façon** sur la clé pour un gabarit que la
  // table plate ne connaît plus — c'est exactement le cas de ceux-ci. Un nom complet est
  // impossible par construction : ce qui le porterait est précisément ce qui a disparu.
  //
  // Et ce qu'on peut faire, à la suite : les refus disent « corriger, puis relancer »,
  // sans quoi le message ne serait qu'un constat de perte. Ici l'axe manquant peut être
  // n'importe lequel des quatre, la phrase dit donc « le catalogue » et non l'imprimeur.
  const pluriel = partis.length > 1;
  box.textContent = box.hidden
    ? ''
    : `${pluriel ? 'Livrables retirés' : 'Livrable retiré'} à l'ouverture, faute de `
      + `catalogue qui ${pluriel ? 'les porte' : 'le porte'} encore : ${partis.join(', ')}. `
      + `Le reste du livre est intact : ${pluriel ? 'les rajouter' : 'le rajouter'} `
      + 'ci-dessous une fois le catalogue complété.';
}

/**
 * La liste des livrables du livre, et de quoi en ajouter un.
 *
 * Une ligne par livrable : ses trois réglages — reliure, finition, papier —, le format
 * de son gabarit, et les relevés que les imprimeurs à gabarit exigent — dos et fond
 * perdu, qu'eux seuls ne publient pas. Plus de cases à cocher : être dans la liste *est*
 * le fait d'être livrable, et rien ne le désigne deux fois.
 *
 * Les trois réglages se construisent sur l'**arbre** du catalogue, seul à savoir ce que
 * ce POD offre ; la table plate ne sert plus qu'au format et au fond perdu, qu'elle
 * seule sait dire.
 *
 * Chaque identifiant de DOM prend la clé du **livrable**, à quatre axes : deux
 * livrables du même gabarit coexistent, et les nommer par le gabarit leur donnerait le
 * même `id`. Toutes les clés du catalogue sont des noms, la clé en est donc un aussi.
 */
function afficherLivrables() {
  const box = $('livrables');
  box.replaceChildren();
  const declares = projet.livraison.livrables;
  for (const d of declares) {
    const p = providers.find((pr) => pr.cle === d.gabarit);
    const pod = pods.find((x) => x.cle === d.pod);
    const ligne = h('div', undefined, 'livrable');
    let releve;
    let raison;
    ligne.append(h('span', libelleProvider(d.gabarit), 'nom'));

    if (pod) {
      // Les reliures du POD, la non outillée grisée : le Rust la refuse déjà en citant
      // sa raison (`catalogue::resout`), et l'écran ne fait que rendre ce refus lisible
      // avant le clic. Le fichier tranche — une reliure porte une géométrie **ou** une
      // raison de ne pas en avoir, jamais les deux.
      const reliure = h('select');
      reliure.id = `liv-reliure-${d.cle}`;
      for (const r of pod.reliures) {
        const o = new Option(r.nom, r.cle);
        o.disabled = r.non_outille !== null;
        reliure.append(o);
      }
      reliure.value = d.reliure;
      // Éteint seulement quand le POD n'a **qu'une** reliure, toutes confondues : un
      // select éteint ne s'ouvre pas, et l'éteindre dès qu'il n'y a qu'une composable
      // cacherait justement le grisé que la spec § 6 demande de montrer — c'est le cas
      // de BoD, le seul POD fourni qui en porte un.
      reliure.disabled = pod.reliures.length < 2;
      reliure.addEventListener('change', () => reglerLivrable(d));
      ligne.append(reliure);

      // La raison, en clair et sur sa propre ligne. Pas une infobulle : c'est la seule
      // partie du message qui distingue « ce POD ne le fait pas » de « l'application ne
      // le compose pas », et elle doit se lire sans survol.
      const grisees = pod.reliures.filter((r) => r.non_outille !== null);
      if (grisees.length) {
        raison = h('p', undefined, 'note raison');
        raison.id = `liv-reliure-raison-${d.cle}`;
        raison.textContent = grisees
          .map((r) => `${r.nom} — ${r.non_outille}`)
          .join(' · ');
      }

      // La finition ne paraît que là où il y en a : un contrôle vide se lit comme un
      // choix qu'on n'a pas su faire, alors qu'il n'y en avait aucun à faire. Le contrôle
      // s'allume donc chez BoD, seul POD fourni à en déclarer — trois pelliculages —, et
      // reste absent chez les cinq autres.
      if (pod.finitions.length) {
        const finition = h('select');
        finition.id = `liv-finition-${d.cle}`;
        // Le vide en tête : aucune finition est le cas courant, et il doit rester
        // choisissable après en avoir pris une.
        finition.append(new Option('—', ''));
        for (const f of pod.finitions) finition.append(new Option(f.nom, f.cle));
        finition.value = d.finition ?? '';
        finition.addEventListener('change', () => reglerLivrable(d));
        ligne.append(finition);
      }

      const papier = h('select');
      papier.id = `liv-papier-${d.cle}`;
      for (const pa of pod.papiers) papier.append(new Option(pa.libelle, pa.cle));
      papier.value = d.papier;
      papier.disabled = pod.papiers.length < 2;
      papier.addEventListener('change', () => reglerLivrable(d));
      ligne.append(papier);

      // Fabriqué ici, avec le POD qui le motive, mais posé après le bouton : le relevé
      // prend une ligne à lui, et l'insérer avant renverrait le format et le bouton
      // « Retirer » au rang suivant, décalés de ceux des voisins. Ordre du balisage et
      // ordre de lecture restent les mêmes — c'est le CSS qui met le relevé à la ligne.
      // Le dos se réclame d'après **le papier retenu**, jamais d'après le POD : un POD
      // peut publier une formule pour l'un de ses papiers et pas pour l'autre.
      const dosPublie = pod.papiers.find((pa) => pa.cle === d.papier)?.dos_publie ?? false;
      if (!dosPublie || p?.fond_perdu === null) {
        releve = h('span', undefined, 'releve');
        const champ = (quoi, libelle, valeur) =>
          releve.append(champReleve(`liv-${quoi}-${d.cle}`, libelle, valeur, d));
        if (!dosPublie) champ('dos', 'Dos relevé (mm)', d.dos_mm);
        if (p?.fond_perdu === null) champ('fp', 'Fond perdu (mm)', d.fond_perdu_mm);
      }
      if (p) ligne.append(h('span', noteFormat(p), 'note'));
    }

    const retirer = h('button', 'Retirer');
    retirer.type = 'button';
    retirer.id = `liv-retirer-${d.cle}`;
    // Le dernier ne se retire pas : le Rust refuse, mais un bouton qui ne peut
    // qu'échouer vaut mieux éteint que refusé.
    retirer.disabled = declares.length < 2;
    retirer.addEventListener('click', () => tente(async () =>
      afficherProjet(await invoke('livrable_retirer', { cle: d.cle }))));
    ligne.append(retirer);
    if (releve) ligne.append(releve);
    if (raison) ligne.append(raison);
    box.append(ligne);
  }

  afficherCascade();
}

/**
 * Les deux listes de l'ajout : le POD, puis **ses** formats.
 *
 * Aucun filtre sur ce qui est déjà déclaré : c'est ce qui permet de déclarer deux fois
 * le même gabarit pour comparer deux papiers. Le vrai doublon — les quatre axes
 * identiques — est refusé par le Rust, avec sa raison.
 *
 * La liste des POD se reconstruit à chaque affichage, celle des formats la suit : elles
 * ne dépendent que du catalogue, qui ne bouge pas de la vie du processus, mais les
 * reconstruire coûte deux boucles sur six entrées et évite d'avoir à se demander qui les
 * a laissées dans quel état.
 */
function afficherCascade() {
  const sel = $('inAjoutPod');
  const choisi = sel.value;
  sel.replaceChildren();
  for (const p of pods) sel.append(new Option(p.nom, p.cle));
  // Le POD retenu survit à un réaffichage : ajouter un livrable ne doit pas ramener la
  // liste sur son premier, alors qu'on en ajoute souvent deux de suite chez le même.
  if (pods.some((p) => p.cle === choisi)) sel.value = choisi;
  sel.disabled = pods.length === 0;
  $('btAjouterLivrable').disabled = pods.length === 0;
  afficherFormatsDuPod();
}

/** Les formats du POD choisi. Vidée et refaite : un format d'un autre POD ne veut rien dire. */
function afficherFormatsDuPod() {
  const p = pods.find((x) => x.cle === $('inAjoutPod').value);
  const sel = $('inAjoutFormat');
  const choisi = sel.value;
  sel.replaceChildren();
  for (const f of p?.formats ?? []) sel.append(new Option(f.nom, f.cle));
  // Le format retenu survit, comme le POD. Comparer deux papiers d'un même livre —
  // le geste pour lequel cet écran existe — c'est déclarer deux fois le même couple
  // imprimeur × format, puis changer le papier sur l'une des deux lignes. Reperdre le
  // format entre les deux ajouts ferait payer deux clics à ce geste-là. Changer de POD
  // l'emporte de lui-même : un format que le nouveau ne porte pas ne se retrouve pas.
  if (p?.formats.some((f) => f.cle === choisi)) sel.value = choisi;
  sel.disabled = !p || p.formats.length < 2;
}

function noteFormat(p) {
  const fp = p.fond_perdu === null
    ? 'fond perdu à relever sur le gabarit'
    : `fond perdu ${nb(p.fond_perdu, 3)} mm`;
  return `${nb(p.largeur, 1)} × ${nb(p.hauteur, 1)} mm — ${fp}`;
}

/**
 * Un relevé fait sur le gabarit de l'imprimeur.
 *
 * Vide au départ, jamais prérempli : un chiffre par défaut se lirait comme une mesure,
 * et une planche composée sur un dos inventé ne se voit qu'au massicot.
 */
function champReleve(id, libelle, valeur, livrable) {
  const l = h('label', undefined, 'petit');
  const i = h('input');
  i.type = 'number';
  i.id = id;
  i.min = 0;
  i.step = 0.1;
  i.value = valeur === null || valeur === undefined ? '' : String(valeur);
  i.addEventListener('change', () => reglerLivrable(livrable));
  l.append(h('span', libelle), i);
  return l;
}

/**
 * Relit la ligne d'un livrable et la renvoie au projet.
 *
 * Le livrable entier voyage, avec les trois axes de son gabarit : régler son papier
 * change son identité, et `cle` dit lequel il était pour que `courant` puisse suivre.
 */
async function reglerLivrable(d) {
  // Un champ vide est une absence de relevé, pas un zéro : composer sur un dos nul
  // produirait une planche fausse au lieu d'un refus.
  const lu = (id) => {
    const v = $(id)?.value.trim();
    return v ? Number(v) : null;
  };
  // Un contrôle absent laisse la valeur qu'il portait : la finition n'a pas de contrôle
  // chez un POD qui n'en déclare aucune, et la ligne ne doit pas l'effacer pour autant.
  const choix = (id, defaut) => $(id)?.value ?? defaut;
  await tente(async () => afficherProjet(await invoke('livrable_regler', {
    cle: d.cle,
    livrable: {
      pod: d.pod,
      format: d.format,
      reliure: choix(`liv-reliure-${d.cle}`, d.reliure),
      papier: choix(`liv-papier-${d.cle}`, d.papier),
      // La chaîne vide du choix « — » est une absence, pas une finition nommée.
      finition: choix(`liv-finition-${d.cle}`, d.finition ?? '') || null,
      dos_mm: lu(`liv-dos-${d.cle}`),
      fond_perdu_mm: lu(`liv-fp-${d.cle}`),
    },
  })));
}

/**
 * Les fichiers d'un package : leur répertoire une fois, leurs noms ensuite.
 *
 * Un package écrit tous ses fichiers au même endroit, et redire soixante-dix caractères
 * de chemin identiques à chaque ligne coûtait deux lignes de plus par livrable —
 * l'ascenseur de la Livraison se payait en redites. Coupés ainsi, les noms tiennent sur
 * une ligne au lieu de se replier au milieu d'un mot.
 *
 * Si les fichiers ne partagent pas leur répertoire, chacun reprend son chemin entier :
 * un chemin long se lit, un chemin faux se suit jusqu'à un fichier qui n'y est pas. Les
 * deux séparateurs sont reconnus — l'application est aussi empaquetée pour Windows, et
 * un `\` pris pour une lettre rendrait le groupement muet là-bas.
 */
function cheminsGroupes(chemins) {
  const dossier = (c) => c.slice(0, Math.max(c.lastIndexOf('/'), c.lastIndexOf('\\')) + 1);
  const commun = chemins.length ? dossier(chemins[0]) : '';
  if (!commun || !chemins.every((c) => dossier(c) === commun)) return chemins;
  return [commun, chemins.map((c) => c.slice(commun.length)).join('   ')];
}

function afficherPackages(resultats) {
  const box = $('packages');
  box.replaceChildren();
  for (const r of resultats) {
    const bloc = h('div', undefined, 'package');
    bloc.append(h('h3', r.libelle));
    if (r.erreur) {
      bloc.append(h('p', r.erreur, 'note alerte'));
    } else {
      const p = r.package;
      const dl = h('dl');
      for (const [k, v] of [
        ['Pages', `${p.pages}${p.blanche ? ' (blanche de parité)' : ''}`],
        ['Papier', p.papier],
        // Après le papier — l'autre chose qu'on choisit sans qu'un octet du PDF change,
        // et qui se commande quand même. La grille alterne les colonnes, les deux ne
        // sont donc pas voisines à l'écran : c'est l'ordre de lecture qui les tient
        // ensemble. Elle ne paraît que là où il y en a une, comme le contrôle de la
        // ligne du livrable, où « aucune » est le cas courant et où une entrée vide se
        // lirait comme un réglage manqué.
        ...(r.finition ? [['Finition', r.finition]] : []),
        ['Gouttière', `${nb(p.gouttiere, 1)} mm`],
        ['Dos', `${nb(p.dos)} mm`],
        ['Planche', `${nb(p.planche[0])} × ${nb(p.planche[1])} mm, `
          + `fond perdu ${nb(p.fond_perdu, 3)} mm`],
      ]) dl.append(h('dt', k), h('dd', v));
      // Les chiffres et les chemins d'un côté, la vignette de l'autre : ce qui
      // s'empilait tient désormais côte à côte, et la hauteur d'un compte rendu est
      // celle de sa planche au lieu d'en être la somme.
      const infos = h('div', undefined, 'infos');
      infos.append(dl);
      // Une police que Typst a remplacée sans échouer : ce PDF-là part chez
      // l'imprimeur, l'alerte se lit donc sur le package qu'elle a traversé.
      // Le dos est composé sur une zone qui rogne ce qui dépasse, sans rien dire : un
      // titre coupé au pli ne se verrait qu'à l'impression. La maquette est unique, les
      // formats ne le sont pas — c'est le seul endroit où ça produit un fichier faux.
      if (p.dos_requis !== null) {
        infos.append(h('p', `Dos de ${nb(p.dos)} mm pour un texte qui en réclame `
          + `${nb(p.dos_requis)} mm : il sera rogné au pli. Réduire le corps du dos, ou `
          + 'y éteindre un élément.', 'note alerte'));
      }
      if (p.polices_introuvables.length) {
        infos.append(h('p', 'Police introuvable, composé dans une écriture de repli : '
          + `${p.polices_introuvables.join(', ')}. Le PDF ne suit pas la maquette.`,
        'note alerte'));
      }
      for (const c of cheminsGroupes(p.chemins)) infos.append(h('p', c, 'chemin'));
      bloc.append(infos);
      // La planche telle qu'elle part à l'impression, avec le dos mesuré de ce
      // livrable-là : c'est ici que « est-ce que ça tient » se vérifie, sur du vrai
      // et non sur une approximation qu'on espère fidèle.
      if (r.vignette) {
        const img = h('img', undefined, 'vignette');
        img.src = r.vignette;
        img.alt = `Planche composée pour ${r.libelle}`;
        bloc.append(img);
      }
    }
    box.append(bloc);
  }
  box.hidden = false;
}

async function packager() {
  const combien = projet.livraison.livrables.length;
  const bt = $('btPackager');
  bt.disabled = true;
  $('packages').hidden = true;
  $('etatPackages').className = 'etat';
  $('etatPackages').textContent = `composition de ${combien} package(s)…`;
  try {
    afficherPackages(await invoke('packager'));
    $('etatPackages').textContent = '';
  } catch (e) {
    $('etatPackages').textContent = String(e);
    $('etatPackages').className = 'etat erreur';
  } finally {
    bt.disabled = false;
  }
}

/** Une taille de fichier, en unités qu'on lit d'un coup d'œil. */
function poids(octets) {
  return octets >= 1024 * 1024
    ? `${nb(octets / (1024 * 1024), 1)} Mo`
    : `${Math.round(octets / 1024)} Ko`;
}

/**
 * Le compte rendu des ebooks : les deux chemins, leur poids, et ce qui s'est passé de
 * travers sans faire échouer la génération.
 *
 * La police non embarquée n'est pas une erreur : le livre reste juste, seul son œil
 * change. Elle se lit donc dans le compte rendu, à côté des chemins, et non en rouge à
 * la place d'un résultat qui existe.
 */
function afficherEbooks(r) {
  const box = $('ebooks');
  box.replaceChildren();
  for (const [chemin, octets] of [[r.pdf, r.octets_pdf], [r.epub, r.octets_epub]]) {
    box.append(h('p', `${chemin}   (${poids(octets)})`, 'chemin'));
  }
  if (r.police_non_embarquee) {
    box.append(h('p', `Police « ${r.police_non_embarquee} » introuvable : l'EPUB est `
      + `dans l'écriture du lecteur. Le texte, lui, est celui du livre.`, 'note'));
  }
  // Celle-ci, en revanche, touche le PDF : c'est le fichier qu'on lira, et il ne suit
  // pas la maquette.
  if (r.polices_introuvables.length) {
    box.append(h('p', 'Police introuvable, composé dans une écriture de repli : '
      + `${r.polices_introuvables.join(', ')}. Le PDF ne suit pas la maquette.`,
    'note alerte'));
  }
  box.hidden = false;
}

async function ebooks() {
  const bt = $('btEbooks');
  bt.disabled = true;
  $('ebooks').hidden = true;
  $('etatEbooks').className = 'etat';
  $('etatEbooks').textContent = 'composition du PDF et de l’EPUB…';
  try {
    afficherEbooks(await invoke('ebook_generer'));
    $('etatEbooks').textContent = '';
  } catch (e) {
    $('etatEbooks').textContent = String(e);
    $('etatEbooks').className = 'etat erreur';
  } finally {
    bt.disabled = false;
  }
}

