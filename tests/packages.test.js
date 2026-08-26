'use strict';

// Câblage de l'étape « Livraison » : ce que l'interface envoie au Rust, et ce qu'elle
// en montre. Le rendu des planches se vérifie dans l'application, pas ici.

const test = require('node:test');
const assert = require('node:assert');
const { charge } = require('./dom_shim');

const LULU = {
  cle: 'lulu-108x175-broche', pod: 'lulu', format: '108x175', reliure: 'broche',
  libelle: 'Lulu — poche 108 × 175',
  largeur: 108, hauteur: 175, fond_perdu: 3.175, dos_publie: true,
  papiers: [{ cle: 'standard', libelle: 'Papier standard' }],
};
const KDP = {
  cle: 'kdp-6x9-broche', pod: 'kdp', format: '6x9', reliure: 'broche',
  libelle: 'Amazon KDP — 6 × 9 po',
  largeur: 152.4, hauteur: 228.6, fond_perdu: 3.175, dos_publie: true,
  papiers: [{ cle: 'creme', libelle: 'Crème' }, { cle: 'blanc', libelle: 'Blanc' }],
};
// Le même imprimeur dans son autre format. La cascade offre les deux, la table plate
// doit donc savoir les dire tous les deux : sans lui, un 5 × 8 ajouté partait au Rust
// sans que rien ne puisse revenir à l'écran, et le test de l'ajout restait vert sur un
// faux qui levait dans le vide.
const KDP_5X8 = {
  cle: 'kdp-5x8-broche', pod: 'kdp', format: '5x8', reliure: 'broche',
  libelle: 'Amazon KDP — 5 × 8 po',
  largeur: 127, hauteur: 203.2, fond_perdu: 3.175, dos_publie: true,
  papiers: [{ cle: 'creme', libelle: 'Crème' }, { cle: 'blanc', libelle: 'Blanc' }],
};
const COOLLIBRI = {
  cle: 'coollibri-148x210-broche', pod: 'coollibri', format: '148x210', reliure: 'broche',
  libelle: 'CoolLibri — A5',
  largeur: 148, hauteur: 210, fond_perdu: null, dos_publie: false,
  papiers: [{ cle: 'mesure', libelle: 'Dos relevé sur le gabarit' }],
};

// L'arbre du catalogue, tel que `pods_liste` le rend. Volontairement plus riche que la
// table plate des tests : c'est lui qui porte les choix, et le grisé motivé n'a rien à
// lire ailleurs.
//
// Chez KDP, la reliure non outillée est posée **avant** la composable : c'est le seul
// ordre qui laisse le test de l'ajout distinguer « la première composable » de « la
// première tout court ». Avec l'ordre inverse, les deux règles rendent la même reliure
// et le test ne protège plus rien.
const PODS = [
  {
    cle: 'lulu', nom: 'Lulu',
    formats: [{ cle: '108x175', nom: 'poche 108 × 175' }],
    reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
    finitions: [],
    papiers: [{ cle: 'standard', libelle: 'Papier standard', teinte: '#ffffff', dos_publie: true }],
  },
  {
    cle: 'kdp', nom: 'Amazon KDP',
    formats: [{ cle: '6x9', nom: '6 × 9 po' }, { cle: '5x8', nom: '5 × 8 po' }],
    reliures: [
      { cle: 'rigide', nom: 'Couverture rigide', non_outille: 'géométrie du casewrap non relevée' },
      { cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null },
    ],
    finitions: [{ cle: 'mat', nom: 'Pelliculage mat' }],
    papiers: [
      { cle: 'creme', libelle: 'Crème', teinte: '#f7f0e0', dos_publie: true },
      { cle: 'blanc', libelle: 'Blanc', teinte: '#ffffff', dos_publie: true },
    ],
  },
  {
    cle: 'coollibri', nom: 'CoolLibri',
    formats: [{ cle: '148x210', nom: 'A5' }],
    reliures: [{ cle: 'broche', nom: 'Broché — dos carré collé', non_outille: null }],
    finitions: [],
    papiers: [{ cle: 'mesure', libelle: 'Dos relevé sur le gabarit', teinte: '#ffffff', dos_publie: false }],
  },
];

/**
 * La face par son libellé, et non par son rang : ces boutons se retrouvent par rang
 * dans l'application — c'est ce que dit le commentaire de `FACES` — mais un test qui
 * en fait autant se met à viser sa voisine le jour où une face s'ajoute. C'est
 * exactement ce qu'a fait l'arrivée du Dos entre la 4ème et la Planche.
 */
const face = (els, libelle) =>
  [...els.get('faces').children].find((b) => b.textContent === libelle);

/**
 * Un livrable neuf chez un prestataire, comme le Rust en fabrique un : les quatre axes
 * à plat, et la clé fabriquée **une fois** ici — le front la reçoit, il ne la recompose
 * jamais.
 */
const chez = (p) => ({
  cle: `${p.pod}-${p.format}-${p.reliure}-${p.papiers[0].cle}`,
  gabarit: p.cle, pod: p.pod, format: p.format, reliure: p.reliure,
  papier: p.papiers[0].cle, finition: null, dos_mm: null, fond_perdu_mm: null,
  compose: null,
});

const PROJET = {
  chemin: '/livres/LHC.ozalid',
  livre: {
    titre: 'Les Heures creuses', titre_page: null, auteur: 'Ivan Pjig',
    genre: 'roman', copyright: '', chapitres: 64,
  },
  manuscrit_absent: false,
  modifie: false,
  manuscrit_source: '/x/WIP7.md',
  chapitres_trouves: 64,
  mots: 49344,
  couverture: null,
  couverture_importee: false,
  images: ['couverture.jpg'],
  interieur: { police: 'Alegreya' },
  envois: { main: { mode: 'police', police: 'Caveat' }, liste: [] },
};

const COMPOSITION = {
  pages: 262, chapitres: 64, gouttiere: 25, blanche: true,
  dos: 16.513, pdf: '/livres/LHC/lulu/interieur-lulu.pdf',
  polices_introuvables: [],
};

/**
 * Le geste qui compose, depuis que le bouton n'existe plus : charger un manuscrit.
 *
 * C'est le consentement du chantier « intérieur sans onglet » — ouvrir un `.ozalid` ne
 * compose pas, charger un manuscrit oui. Les tests qui ont besoin d'un livre composé
 * passent donc par là, comme l'utilisateur.
 *
 * `manuscritRemplace` lance la composition sans l'attendre — l'utilisateur non plus.
 * Un tour de boucle pour qu'elle aboutisse avant qu'on regarde le résultat.
 */
const faireComposer = async (els) => {
  await els.get('btReimporter').declenche('click');
  await new Promise((r) => setImmediate(r));
};

function paquet(sur = {}) {
  return {
    cle: 'lulu-108x175-broche-standard',
    libelle: 'Lulu — poche 108 × 175',
    papier: 'Papier standard',
    pages: 262,
    gouttiere: 25,
    blanche: true,
    dos: 16.513,
    dos_requis: null,
    fond_perdu: 3.175,
    planche: [238.863, 181.35],
    chemins: ['/livres/LHC/lulu/interieur-lulu.pdf', '/livres/LHC/lulu/couverture-lulu.pdf'],
    vignette: '/livres/LHC/lulu/couverture-lulu.png',
    polices_introuvables: [],
    ...sur,
  };
}

/**
 * Un projet ouvert, avec un Rust de façade qui **tient réellement** la liste des
 * destinataires.
 *
 * Depuis le lot 3, le prestataire vit dans le projet et non dans un contrôle : le front
 * relit la liste à chaque retour de commande. Un faux qui rendrait toujours le même
 * projet ne prouverait donc plus rien — il masquerait justement le câblage qu'on vérifie.
 */
async function ouvre(
  providers,
  sur = {},
  { couverture = null, destinataires, dejaCompose = false, dosParPapier = {}, pods = [] } = {}
) {
  const appels = [];
  const liste = (destinataires ?? [chez(providers[0])]).map((d) => ({ ...d }));
  let projet = {
    ...PROJET,
    couverture,
    livraison: { livrables: liste, courant: liste[0].cle, deja_compose: dejaCompose },
  };
  const maj = (livraison) => {
    projet = { ...projet, livraison: { ...projet.livraison, ...livraison } };
    return projet;
  };
  // Les règles du Rust, modélisées ici : la mesure d'une composition entre chez le
  // destinataire pour qui elle a été faite, et tout ce qui pagine les efface toutes.
  // Sans ce modèle, le front n'aurait plus rien à lire — il ne tient plus de dos.
  const oublier = () => maj({
    livrables: projet.livraison.livrables.map(({ compose, ...d }) => d),
  });
  // La mesure entre chez tous les livrables du **gabarit** composé : c'est là qu'elle
  // vit désormais, et c'est ce partage qui rend la comparaison de deux papiers gratuite.
  const retenir = (c) => maj({
    deja_compose: true,
    livrables: projet.livraison.livrables.map((d) => (
      d.gabarit === projet.livraison.livrables.find((x) => x.cle === projet.livraison.courant)?.gabarit
        ? {
          ...d,
          compose: {
            pages: c.pages, gouttiere: c.gouttiere, blanche: c.blanche, dos: c.dos,
          },
        }
        : d
    )),
  });
  const invoke = async (cmd, args) => {
    appels.push([cmd, args]);
    if (cmd in sur) {
      const v = sur[cmd];
      const r = typeof v === 'function' ? await v(args) : v;
      // Une composition surchargée par un test reste soumise aux règles : c'est le
      // projet qui porte la mesure, et le front la relit là.
      return cmd === 'composer' ? { ...r, projet: retenir(r) } : r;
    }
    if (cmd === 'providers_liste') return providers;
    if (cmd === 'pods_liste') return pods;
    if (cmd === 'catalogue_refus') return [];
    if (cmd === 'polices_liste') return ['Archivo', 'Spectral'];
    if (cmd === 'polices_texte_liste') return ['EB Garamond', 'Alegreya', 'Cardo'];
    if (cmd === 'jetons_liste') return ['%TITRE%', '%AUTEUR%', '%GENRE%', '%EDITEUR%', '%COLLECTION%', '%MONOGRAMME%'];
    if (cmd === 'mains_liste') return ['Caveat', 'Dancing Script'];
    if (cmd === 'maquettes_liste') return [{ cle: 'bandeau', libelle: 'Bandeau' }];
    if (cmd === 'projet_ouvrir') return projet;
    if (cmd === 'couverture_apercu') return { image: 'data:image/png;base64,QUJD', reperes: null };
    if (cmd === 'livrable_viser') return maj({ courant: args.cle });
    if (cmd === 'livrable_regler') {
      // Le Rust recalcule le dos à la vue, depuis la formule du papier retenu. Le faux
      // n'a pas les formules : il sert celui que le test lui donne pour ce papier-là.
      const redos = (d) => (dosParPapier[d.papier] === undefined || !d.compose
        ? d
        : { ...d, compose: { ...d.compose, dos: dosParPapier[d.papier] } });
      // Le papier change l'identité du livrable, jamais son gabarit : la mesure vit
      // sous le gabarit, et **survit** au réglage. Le faux ne la touche donc plus.
      const l = args.livrable;
      const neuve = `${l.pod}-${l.format}-${l.reliure}-${l.papier}`;
      const majee = maj({
        livrables: projet.livraison.livrables.map((d) => (
          d.cle === args.cle ? redos({ ...d, ...l, cle: neuve }) : d
        )),
      });
      return projet.livraison.courant === args.cle ? maj({ courant: neuve }) : majee;
    }
    if (cmd === 'livrable_ajouter') {
      const f = args.fabrication;
      const p = providers.find((x) => x.cle === `${f.pod}-${f.format}-${f.reliure}`);
      // La règle du Rust : le refus porte sur les quatre axes, et sur eux seuls.
      const neuve = `${p.cle}-${f.papier}`;
      if (projet.livraison.livrables.some((d) => d.cle === neuve)) {
        throw new Error(`${p.libelle} en ${f.papier} est déjà un livrable de ce livre.`);
      }
      return maj({
        livrables: [
          ...projet.livraison.livrables,
          { ...chez(p), papier: f.papier, cle: neuve },
        ],
      });
    }
    if (cmd === 'livrable_retirer') {
      return maj({
        livrables: projet.livraison.livrables.filter((d) => d.cle !== args.cle),
      });
    }
    if (cmd === 'interieur_modifier') {
      projet = { ...projet, interieur: args.interieur };
      return oublier();
    }
    if (cmd === 'livre_modifier') {
      projet = { ...projet, livre: args.livre };
      return oublier();
    }
    if (cmd === 'manuscrit_reimporter' || cmd === 'manuscrit_choisir') return oublier();
    // Le démarrage et la garde envoient ces trois commandes sans qu'aucun test ne les
    // demande : sans réponse ici, elles lèveraient avant que rien ne soit vérifié.
    if (cmd === 'recents_liste') return [];
    if (cmd === 'garde_modifications') return 'ignorer';
    if (cmd === 'interface_prete') return null;
    // L'accès au modèle de diffusion se lit au démarrage : il appartient à la
    // machine, et l'écran le montre avant qu'aucun projet ne soit ouvert.
    if (cmd === 'diffusion_lire') return { url: '', modele: '', cle_posee: false };
    throw new Error(`commande inattendue : ${cmd}`);
  };
  const ctx = await charge({ invoke, open: async () => '/livres/LHC.ozalid' });
  await ctx.els.get('btOuvrir').declenche('click');
  return { ...ctx, appels };
}

const attendreApercu = () => new Promise((r) => setTimeout(r, 300));
/** Plus long que le débounce de la recomposition automatique (400 ms). */
const attendreComposition = () => new Promise((r) => setTimeout(r, 700));
const combien = (appels, cmd) => appels.filter(([c]) => c === cmd).length;
const dernier = (appels, cmd) => appels.filter(([c]) => c === cmd).pop();

/* ---------- la liste des destinataires ---------- */

/**
 * Un prestataire qui publie sa formule n'a rien à faire saisir : offrir un champ de
 * dos donnerait à croire qu'il compte, alors que la formule prime toujours.
 */
test('seul un prestataire à gabarit demande un relevé', async () => {
  const { els } = await ouvre([LULU, COOLLIBRI], {}, {
    destinataires: [chez(LULU), chez(COOLLIBRI)],
  });
  assert.ok(!els.get('dest-dos-lulu-108x175-broche-standard'), 'dos saisissable chez Lulu');
  assert.ok(!els.get('dest-fp-lulu-108x175-broche-standard'), 'fond perdu saisissable chez Lulu');
  assert.ok(els.get('dest-dos-coollibri-148x210-broche-mesure'), 'dos non demandé chez CoolLibri');
  assert.ok(els.get('dest-fp-coollibri-148x210-broche-mesure'), 'fond perdu non demandé chez CoolLibri');
});

/**
 * La liste ne montre que les destinataires du livre — c'est tout l'objet du lot : un
 * prestataire n'est plus désigné deux fois, et la table entière n'a plus à s'afficher.
 */
test('la liste ne porte que les destinataires déclarés', async () => {
  const { els } = await ouvre([LULU, KDP, COOLLIBRI], {}, { destinataires: [chez(LULU)] });
  assert.deepStrictEqual(els.get('destinataires').textes('span').filter((t) => t.includes('—')), [
    'Lulu — poche 108 × 175',
    '108,0 × 175,0 mm — fond perdu 3,175 mm',
  ]);
  assert.ok(!els.get('dest-papier-kdp-6x9-broche-creme'), 'un prestataire non destinataire est offert');
});

/**
 * La liste d'ajout **ne filtre plus** ce qui est déjà déclaré : c'est ce qui permet de
 * déclarer deux fois le même gabarit pour comparer deux papiers. Ce qui est refusé,
 * c'est le vrai doublon — les quatre axes —, et c'est le Rust qui le refuse, en le
 * disant.
 */
test('la liste d\'ajout garde les gabarits déjà déclarés', async () => {
  const { els, appels } = await ouvre([LULU, KDP, COOLLIBRI], {}, {
    destinataires: [chez(LULU), chez(KDP)], pods: PODS,
  });
  assert.deepStrictEqual(
    els.get('inAjoutPod').textes('option'),
    ['Lulu', 'Amazon KDP', 'CoolLibri']
  );

  els.get('inAjoutPod').value = 'coollibri';
  await els.get('inAjoutPod').declenche('change');
  await els.get('btAjouterDestinataire').declenche('click');
  assert.ok(els.get('dest-papier-coollibri-148x210-broche-mesure'), 'ajout sans effet à l\'écran');
  // La fabrication entière part au Rust, pas une clé à découper : les trois axes du
  // gabarit viennent de la table, le papier est celui d'office du POD.
  assert.deepStrictEqual({ ...dernier(appels, 'livrable_ajouter')[1].fabrication }, {
    pod: 'coollibri', format: '148x210', reliure: 'broche', papier: 'mesure',
  });
  assert.strictEqual(
    els.get('btAjouterDestinataire').disabled,
    false,
    'ajouter s\'est éteint : la table n\'est jamais épuisée'
  );
});

/**
 * Le pendant du filtre disparu : déclarer deux fois les mêmes quatre axes écrirait les
 * mêmes octets dans deux répertoires. Le Rust refuse, et le refus doit se lire — c'est
 * la seule chose qui reste à l'écran pour dire pourquoi rien ne s'est ajouté.
 */
test('le même livrable deux fois est refusé, et le refus se lit', async () => {
  const { els } = await ouvre([LULU, KDP], {}, { destinataires: [chez(LULU)], pods: PODS });

  els.get('inAjoutPod').value = 'lulu';
  await els.get('inAjoutPod').declenche('change');
  await els.get('btAjouterDestinataire').declenche('click');

  assert.match(els.get('alerte').textContent, /déjà un livrable/);
  assert.strictEqual(els.get('destinataires').children.length, 1,
    'le doublon s\'est ajouté malgré le refus');
});

test('la cascade offre les formats du POD choisi, et eux seuls', async () => {
  const { els } = await ouvre([LULU, KDP, COOLLIBRI], {}, { pods: PODS });

  assert.deepStrictEqual(
    els.get('inAjoutPod').textes('option'),
    ['Lulu', 'Amazon KDP', 'CoolLibri'],
    'la liste des POD ne les donne pas tous, ou pas dans l\'ordre du catalogue'
  );
  // Le premier POD est choisi d'office : une cascade qui commence vide demande un clic
  // pour ne rien dire.
  assert.deepStrictEqual(els.get('inAjoutFormat').textes('option'), ['poche 108 × 175']);

  els.get('inAjoutPod').value = 'kdp';
  await els.get('inAjoutPod').declenche('change');
  assert.deepStrictEqual(
    els.get('inAjoutFormat').textes('option'),
    ['6 × 9 po', '5 × 8 po'],
    'changer de POD n\'a pas rechargé ses formats'
  );
});

test('l\'ajout envoie les quatre axes, la reliure composable et le premier papier', async () => {
  const { els, appels } = await ouvre([LULU, KDP, KDP_5X8, COOLLIBRI], {}, { pods: PODS });

  els.get('inAjoutPod').value = 'kdp';
  await els.get('inAjoutPod').declenche('change');
  els.get('inAjoutFormat').value = '5x8';
  await els.get('btAjouterDestinataire').declenche('click');

  const [, args] = appels.findLast(([cmd]) => cmd === 'livrable_ajouter');
  // Étalé : la fabrication vient du contexte du front, et `deepStrictEqual` compare
  // aussi les prototypes — c'est ce que fait déjà l'autre test de l'ajout, plus haut.
  assert.deepStrictEqual({ ...args.fabrication }, {
    // La reliure d'office est la première **composable** : la rigide de KDP porte une
    // raison de ne pas l'être, et le Rust la refuserait.
    pod: 'kdp', format: '5x8', reliure: 'broche', papier: 'creme',
  });
  // L'ajout doit **aboutir**, pas seulement partir. Sans cette ligne, le test lisait le
  // départ de la commande et rien d'autre : il serait resté vert sur une fabrication que
  // le Rust refuse, l'exception étant avalée par `tente()`.
  assert.ok(
    els.get('dest-papier-kdp-5x8-broche-creme'),
    'le livrable ajouté ne paraît pas à l\'écran'
  );
  // Le format retenu survit à l'ajout, comme le POD : comparer deux papiers d'un même
  // livre, ce que cet écran existe pour permettre, c'est ajouter deux fois le même
  // couple imprimeur × format avant de changer le papier sur l'une des deux lignes.
  assert.strictEqual(
    els.get('inAjoutFormat').value,
    '5x8',
    'le format est retombé sur le premier du POD entre deux ajouts'
  );
});

/**
 * **La promesse du lot, à l'écran.** Deux livrables du même gabarit coexistent, et rien
 * ne les confond : leurs lignes portent des `id` distincts — c'est la clé à quatre axes
 * qui les nomme, jamais le gabarit, que les deux partagent — et le sélecteur du pied les
 * donne à lire distincts, le papier étant tout ce qui les sépare.
 *
 * Sans cela, les deux lignes s'écraseraient l'une l'autre dans le document : régler le
 * papier de la première irait lire le champ de la seconde, et le refus du Rust serait le
 * seul à s'en apercevoir.
 */
test('deux papiers d\'un même gabarit tiennent deux lignes distinctes', async () => {
  const { els } = await ouvre([KDP], {}, {
    destinataires: [
      chez(KDP),
      { ...chez(KDP), papier: 'blanc', cle: 'kdp-6x9-broche-blanc' },
    ],
  });

  assert.ok(els.get('dest-papier-kdp-6x9-broche-creme'), 'la ligne du crème manque');
  assert.ok(els.get('dest-papier-kdp-6x9-broche-blanc'), 'la ligne du blanc manque');
  assert.strictEqual(els.get('dest-papier-kdp-6x9-broche-creme').value, 'creme');
  assert.strictEqual(els.get('dest-papier-kdp-6x9-broche-blanc').value, 'blanc');
  assert.notStrictEqual(
    els.get('dest-retirer-kdp-6x9-broche-creme'),
    els.get('dest-retirer-kdp-6x9-broche-blanc'),
    'les deux lignes ne font qu\'un bouton : les `id` sont fabriqués sur le gabarit'
  );
  // Le pied doit les nommer distinctement : c'est là qu'on choisit lequel on compose,
  // et deux libellés identiques ne se choisiraient pas.
  assert.deepStrictEqual(els.get('inDestinataire').textes('option'), [
    'Amazon KDP — 6 × 9 po — Crème',
    'Amazon KDP — 6 × 9 po — Blanc',
  ]);
});

/**
 * Le dernier destinataire ne se retire pas : c'est lui qui donne son format à l'aperçu,
 * et une liste vide rendrait la Couverture inutilisable. Le Rust refuse ; le bouton
 * s'éteint plutôt que de mener à ce refus.
 */
test('le dernier destinataire ne peut pas être retiré', async () => {
  const { els, appels } = await ouvre([LULU, KDP], {}, {
    destinataires: [chez(LULU), chez(KDP)],
  });
  assert.strictEqual(els.get('dest-retirer-lulu-108x175-broche-standard').disabled, false);

  await els.get('dest-retirer-kdp-6x9-broche-creme').declenche('click');
  assert.strictEqual(dernier(appels, 'livrable_retirer')[1].cle, 'kdp-6x9-broche-creme');
  assert.strictEqual(
    els.get('dest-retirer-lulu-108x175-broche-standard').disabled,
    true,
    'le dernier destinataire reste retirable'
  );
});

/* ---------- les relevés ---------- */

test('un relevé saisi part au projet, avec le papier de la ligne', async () => {
  const { els, appels } = await ouvre([COOLLIBRI], {}, { destinataires: [chez(COOLLIBRI)] });

  els.get('dest-dos-coollibri-148x210-broche-mesure').value = '18.4';
  els.get('dest-fp-coollibri-148x210-broche-mesure').value = '4';
  await els.get('dest-dos-coollibri-148x210-broche-mesure').declenche('change');

  // Le livrable entier voyage, ses trois axes de gabarit compris : régler son papier
  // change son identité, et `cle` dit lequel il était pour que `courant` puisse suivre.
  const [, args] = dernier(appels, 'livrable_regler');
  assert.strictEqual(args.cle, 'coollibri-148x210-broche-mesure');
  // Étalé : l'objet vient du contexte `vm`, et `deepStrictEqual` compare les prototypes.
  assert.deepStrictEqual({ ...args.livrable }, {
    pod: 'coollibri',
    format: '148x210',
    reliure: 'broche',
    papier: 'mesure',
    finition: null,
    dos_mm: 18.4,
    fond_perdu_mm: 4,
  });
});

/**
 * Un champ vidé est une absence de relevé, pas un zéro. La différence n'est pas
 * cosmétique : un dos de zéro millimètre compose une planche que rien ne refuse, et
 * qui ne se voit qu'au massicot. Un relevé absent, lui, fait refuser la composition.
 */
test('un relevé effacé redevient une absence, jamais un zéro', async () => {
  const { els, appels } = await ouvre([COOLLIBRI], {}, {
    destinataires: [{ ...chez(COOLLIBRI), dos_mm: 18.4, fond_perdu_mm: 4 }],
  });
  assert.strictEqual(els.get('dest-dos-coollibri-148x210-broche-mesure').value, '18.4');

  els.get('dest-dos-coollibri-148x210-broche-mesure').value = '';
  await els.get('dest-dos-coollibri-148x210-broche-mesure').declenche('change');

  assert.strictEqual(dernier(appels, 'livrable_regler')[1].livrable.dos_mm, null);
});

/* ---------- génération ---------- */

/**
 * La génération n'envoie plus rien : la liste est dans le projet. Lui repasser des
 * cases cochées rétablirait la double désignation que ce lot supprime.
 */
test('générer ne transmet aucune liste : elle est dans le projet', async () => {
  const { els, appels } = await ouvre([LULU, KDP], {
    packager: () => [{ cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), erreur: null }],
  }, { destinataires: [chez(LULU), chez(KDP)] });

  await els.get('btPackager').declenche('click');
  assert.deepStrictEqual(dernier(appels, 'packager')[1], undefined);
  assert.match(els.get('packages').textContent, /16,51 mm/);
});

/**
 * Un prestataire en échec ne doit pas emporter les autres : ce qui a été produit est
 * livrable, et l'échec doit être lisible plutôt que noyé dans un message global.
 */
test('un prestataire en échec est signalé sans masquer ceux qui ont abouti', async () => {
  const { els } = await ouvre([LULU, KDP], {
    packager: () => [
      { cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), vignette: null, erreur: null },
      {
        cle: 'kdp-6x9-broche-creme',
        libelle: 'Amazon KDP',
        package: null,
        vignette: null,
        erreur: '1200 pages : tranche de gouttière absente du gabarit kdp-6x9',
      },
    ],
  }, { destinataires: [chez(LULU), chez(KDP)] });
  await els.get('btPackager').declenche('click');

  const box = els.get('packages');
  assert.strictEqual(box.hidden, false);
  assert.deepStrictEqual(box.textes('h3'), ['Lulu', 'Amazon KDP']);
  assert.match(box.textContent, /16,51 mm/, 'dos du package abouti absent');
  assert.match(box.textContent, /tranche de gouttière absente/);
});

/**
 * Même promesse que pour la composition : une police que Typst a remplacée sans
 * échouer doit se lire sur le package qu'elle a traversé — c'est ce PDF-là qui part
 * chez l'imprimeur.
 */
test('un package composé par repli porte l\'alerte de police', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{
      cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', erreur: null,
      package: paquet({ polices_introuvables: ['plume ivan'] }),
    }],
  });
  await els.get('btPackager').declenche('click');

  const t = els.get('packages').textContent;
  assert.match(t, /plume ivan/);
  assert.match(t, /repli/);
});

test('un package sans substitution n\'affiche aucune alerte de police', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{ cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), erreur: null }],
  });
  await els.get('btPackager').declenche('click');
  assert.doesNotMatch(els.get('packages').textContent, /repli/);
});

/**
 * Le seul endroit où une maquette unique pour N formats produit un fichier **faux** et
 * non un fichier différent : le corps du dos suit la largeur de couverture, son
 * épaisseur suit la pagination, et la zone qui compose le dos rogne ce qui dépasse sans
 * rien dire. Le compte rendu du package est le dernier écran avant l'imprimeur.
 */
test('un dos trop mince pour son texte porte l\'alerte sur son package', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{
      cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', erreur: null,
      package: paquet({ dos: 4.2, dos_requis: 6.31 }),
    }],
  });
  await els.get('btPackager').declenche('click');

  const t = els.get('packages').textContent;
  assert.match(t, /4,20 mm/, 'le dos réel doit se lire');
  assert.match(t, /6,31 mm/, 'le dos réclamé doit se lire');
  assert.match(t, /rogné/);
});

test('un dos qui tient n\'affiche aucune alerte', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{ cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), erreur: null }],
  });
  await els.get('btPackager').declenche('click');
  assert.doesNotMatch(els.get('packages').textContent, /rogné/);
});

test('un package affiche le dos, la planche et les fichiers produits', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{
      cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), vignette: null, erreur: null,
    }],
  });
  await els.get('btPackager').declenche('click');

  const dd = els.get('packages').textes('dd');
  assert.deepStrictEqual(dd, [
    '262 (blanche de parité)',
    'Papier standard',
    '25,0 mm',
    '16,51 mm',
    '238,86 × 181,35 mm, fond perdu 3,175 mm',
  ]);
  assert.match(els.get('packages').textContent, /couverture-lulu\.pdf/);
});

/**
 * Le répertoire une fois, les noms ensuite. Ce n'est pas de la cosmétique : le compte
 * rendu de deux destinataires ne tient dans la fenêtre que si le chemin du projet n'y
 * est pas écrit quatre fois. Ce que le test protège, c'est que les noms de fichiers
 * restent lisibles — pas la mise en page qui les range.
 */
test('les fichiers d\'un package nomment leur répertoire une seule fois', async () => {
  const { els } = await ouvre([LULU], {
    packager: () => [{
      cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), vignette: null, erreur: null,
    }],
  });
  await els.get('btPackager').declenche('click');

  const lignes = els.get('packages').textes('p');
  assert.deepStrictEqual(lignes, [
    '/livres/LHC/lulu/',
    'interieur-lulu.pdf   couverture-lulu.pdf',
  ]);
});

/**
 * Deux fichiers dans deux répertoires n'ont pas de répertoire commun à mettre en
 * facteur : chacun reprend le sien, en entier. Un chemin long se lit ; un chemin
 * raccourci de travers se suit jusqu'à un fichier qui n'existe pas.
 */
test('des fichiers dispersés gardent chacun leur chemin entier', async () => {
  const disperses = { ...paquet(), chemins: ['/a/interieur.pdf', '/b/couverture.pdf'] };
  const { els } = await ouvre([LULU], {
    packager: () => [{
      cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: disperses, vignette: null, erreur: null,
    }],
  });
  await els.get('btPackager').declenche('click');

  assert.deepStrictEqual(els.get('packages').textes('p'),
    ['/a/interieur.pdf', '/b/couverture.pdf']);
});

/**
 * La vignette est le seul endroit où « est-ce que ça tient » se vérifie sur du vrai,
 * pour chaque prestataire, avec son dos mesuré. Le package qui a échoué n'en a pas —
 * et l'absence ne doit pas poser une image vide, qui se lirait comme une planche.
 */
test('chaque package abouti montre sa planche en vignette', async () => {
  const { els } = await ouvre([LULU, KDP], {
    packager: () => [
      {
        cle: 'lulu-108x175-broche-standard',
        libelle: 'Lulu',
        package: paquet(),
        vignette: 'data:image/png;base64,QUJD',
        erreur: null,
      },
      {
        cle: 'kdp-6x9-broche-creme', libelle: 'KDP', package: null, vignette: null, erreur: 'raté',
      },
    ],
  }, { destinataires: [chez(LULU), chez(KDP)] });
  await els.get('btPackager').declenche('click');

  const images = [];
  const visite = (e) => {
    if (e.tagName === 'IMG') images.push(e);
    e.enfants.forEach(visite);
  };
  els.get('packages').enfants.forEach(visite);
  assert.strictEqual(images.length, 1, 'une vignette pour un package en échec');
  assert.strictEqual(images[0].src, 'data:image/png;base64,QUJD');
});

/* ---------- aperçu de la planche ---------- */

/**
 * Le cœur du projet, vu de l'interface : le dos de l'aperçu vient de la composition,
 * jamais d'une saisie. Tant que l'intérieur n'a pas été composé, il n'y a pas de dos
 * à passer — et la planche refusera de s'afficher plutôt que d'en inventer un.
 */
test('l\'aperçu de planche n\'a pas de dos tant que l\'intérieur n\'est pas composé', async () => {
  const { els, appels } = await ouvre([LULU], {}, { couverture: {} });
  await face(els, 'Planche').declenche('click');
  await attendreApercu();

  const [, args] = dernier(appels, 'couverture_apercu');
  assert.strictEqual(args.face, 'planche');
  assert.strictEqual(args.dosMm, null, 'un dos est passé sans composition');
});

/**
 * Le gabarit ne voyage plus avec l'aperçu : le Rust le lit dans le projet. Le repasser
 * ici rouvrirait la porte à deux vérités sur le prestataire courant.
 */
test('l\'aperçu ne transporte plus de gabarit', async () => {
  const { els, appels } = await ouvre([LULU], {}, { couverture: {} });
  await face(els, '1ère').declenche('click');
  await attendreApercu();

  assert.deepStrictEqual(
    Object.keys(dernier(appels, 'couverture_apercu')[1]).sort(),
    ['dosMm', 'face']
  );
});

test('une fois l\'intérieur composé, l\'aperçu de planche reçoit ce dos-là', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();

  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);
});

/** Composer, c'est composer pour le destinataire visé : plus rien à lui désigner. */
test('composer ne transmet plus de prestataire', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION });
  await faireComposer(els);
  assert.deepStrictEqual(dernier(appels, 'composer')[1], undefined);
});

/**
 * Le dos vaut pour un gabarit et un seul : le même manuscrit ne fait pas le même
 * nombre de pages en poche et en grand format. Le traîner d'un destinataire à l'autre
 * donnerait à voir une planche fausse, et c'est exactement le défaut que l'atelier
 * HTML avait.
 */
test('viser un autre destinataire périme le dos de l\'aperçu', async () => {
  const { els, appels } = await ouvre([LULU, KDP], { composer: COMPOSITION }, {
    couverture: {}, destinataires: [chez(LULU), chez(KDP)],
  });
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);

  els.get('inDestinataire').value = 'kdp-6x9-broche-creme';
  await els.get('inDestinataire').declenche('change');
  await attendreApercu();
  assert.strictEqual(
    dernier(appels, 'couverture_apercu')[1].dosMm,
    null,
    'dos de Lulu réutilisé pour KDP'
  );
});

/**
 * Le papier déplace le dos **sans passer par la pagination** : chez KDP, 0,0635 mm par
 * page en crème contre 0,0572 en blanc, soit 1,65 mm d'écart sur 262 pages —
 * l'épaisseur d'une couverture entière.
 *
 * Ce n'est plus une péremption depuis que la mesure vit sous le gabarit : les deux
 * papiers la partagent, et chacun en tire son dos. La planche doit donc montrer
 * aussitôt celui du blanc — et **sans recomposer**, c'est toute la promesse du lot.
 * Le chiffre est recalculé par le Rust ; ce qui se vérifie ici, c'est que le pointeur a
 * suivi l'identité du livrable réglé, faute de quoi l'aperçu n'aurait plus de dos du
 * tout.
 */
test('changer de papier montre le dos de ce papier, sans recomposer', async () => {
  const { els, appels } = await ouvre([KDP], { composer: COMPOSITION }, {
    couverture: {}, dosParPapier: { blanc: 14.986 },
  });
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);
  const avant = combien(appels, 'composer');

  els.get('dest-papier-kdp-6x9-broche-creme').value = 'blanc';
  await els.get('dest-papier-kdp-6x9-broche-creme').declenche('change');
  await attendreComposition();
  await attendreApercu();

  assert.strictEqual(
    dernier(appels, 'couverture_apercu')[1].dosMm,
    14.986,
    'dos du papier crème réutilisé pour le blanc, ou pointeur perdu en chemin'
  );
  assert.strictEqual(combien(appels, 'composer'), avant,
    'comparer deux papiers a coûté une composition');
});

/**
 * Même raison, autre cause : la police repagine le livre. Un dos calculé en Alegreya
 * n'est plus le dos du livre dès qu'on le compose en Cardo, et le laisser sur la
 * planche donnerait un chiffre faux — ce qui vaut moins que pas de chiffre.
 */
test('un dos calculé pour une autre police ne vaut plus rien', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);

  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  await attendreApercu();
  assert.strictEqual(
    dernier(appels, 'couverture_apercu')[1].dosMm,
    null,
    'dos d\'Alegreya réutilisé pour Cardo'
  );
});

/**
 * **Le test qui porte le lot.** Le même livre a autant de paginations que de gabarits,
 * et chacune coûte une composition entière. Les retenir une par destinataire, dans le
 * projet, fait de la lunette ce qu'elle prétend être : revenir sur un prestataire déjà
 * composé retrouve son dos, sans rien recalculer et sans emprunter celui du voisin.
 *
 * Le compte des `composer` est la moitié du test : sans lui, une implémentation qui
 * recomposerait en douce à chaque aller-retour passerait pour juste.
 */
test('revenir à un destinataire déjà composé retrouve son dos sans recomposer', async () => {
  const dos = [16.513, 21.4];
  let n = 0;
  const { els, appels } = await ouvre([LULU, KDP], {
    composer: () => ({ ...COMPOSITION, dos: dos[n++] }),
  }, { couverture: {}, destinataires: [chez(LULU), chez(KDP)] });
  const vise = async (cle) => {
    els.get('inDestinataire').value = cle;
    await els.get('inDestinataire').declenche('change');
    await attendreComposition();
  };
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);

  // KDP n'a jamais été composé : la veille s'en charge, et lui donne son dos à lui.
  await vise('kdp-6x9-broche-creme');
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 21.4);

  const avant = combien(appels, 'composer');
  await vise('lulu-108x175-broche-standard');
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513,
    'le dos de Lulu n\'a pas été retrouvé');
  assert.strictEqual(combien(appels, 'composer'), avant,
    'revenir sur un destinataire déjà composé a recomposé');
});

/**
 * Ce que le lot rend : une mesure périmée se refait toute seule. Le geste qui l'a
 * périmée — ici la police — suffit, et le bouton n'est plus qu'un recours.
 */
test('une modification recompose d\'elle-même, une fois le livre composé', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await faireComposer(els);
  assert.strictEqual(combien(appels, 'composer'), 1);

  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  await attendreComposition();

  assert.strictEqual(combien(appels, 'composer'), 2, 'la police n\'a rien relancé');
});

/**
 * L'autre moitié de la règle, et la plus importante : **ouvrir n'est pas demander**.
 * Une composition dure des secondes et écrit des fichiers ; la déclencher chez
 * quelqu'un qui a seulement ouvert un `.ozalid` — pour regarder une couverture, par
 * exemple — coûterait bien plus que ce qu'on lui épargne.
 *
 * C'est le pari du chantier « intérieur sans onglet » : le consentement a quitté le
 * bouton, qui n'existe plus, pour le chargement d'un manuscrit. Ce test est celui qui
 * le garde — sans lui, rien n'empêcherait de faire tourner Typst à chaque ouverture.
 */
test('ouvrir un projet ne compose pas', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });

  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  els.get('inDedicace').value = 'À M.';
  await els.get('inDedicace').declenche('change');
  await attendreComposition();

  assert.strictEqual(combien(appels, 'composer'), 0, 'composé sans qu\'on le demande');
});

/**
 * Et l'autre face : charger un manuscrit compose, **sans qu'on ait rien cliqué d'autre**.
 * C'est le geste qui dit « ce livre m'intéresse », et il n'y a plus de bouton derrière.
 */
test('charger un manuscrit compose', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  assert.strictEqual(combien(appels, 'composer'), 0, 'composé avant le manuscrit');

  await faireComposer(els);

  assert.strictEqual(combien(appels, 'composer'), 1, 'le manuscrit n\'a rien déclenché');
});

/**
 * **Le test du consentement de session.** Si la toute première composition échoue,
 * `deja_compose` reste faux dans le projet — il ne se lève qu'à une réussite. Sans une
 * mémoire côté écran, corriger la cause ne relancerait rien, et il n'y a plus de bouton
 * pour reprendre : on serait devant une impasse.
 *
 * C'est le trou que l'écriture du plan a trouvé, et le seul correctif de ce lot.
 */
test('un premier échec n\'empêche pas la reprise quand on corrige la cause', async () => {
  let echoue = true;
  const { els, appels } = await ouvre([LULU], {
    composer: () => {
      if (echoue) throw 'police d\'intérieur inconnue';
      return COMPOSITION;
    },
  }, { couverture: {} });

  await faireComposer(els);
  assert.strictEqual(combien(appels, 'composer'), 1);
  assert.match(els.get('alerte').textContent, /police d'intérieur inconnue/);

  // On corrige la cause. Rien d'autre ne se passe : aucun bouton, et le projet ne porte
  // toujours aucune mesure ni aucun `deja_compose`.
  echoue = false;
  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  await attendreComposition();

  assert.strictEqual(combien(appels, 'composer'), 2,
    'la correction n\'a rien relancé : le livre est dans une impasse');
});

/**
 * Une composition dure des dizaines de secondes sur un vrai manuscrit, et personne ne
 * l'a demandée. Le pied doit dire qu'elle tourne — laisser « dos périmé » en rouge tout
 * ce temps ferait lire une panne là où il n'y a qu'un travail en cours. C'est le mot que
 * `#etat` disait à côté du bouton, déménagé où le compte rendu vit désormais.
 */
test('le pied dit que la composition tourne', async () => {
  const { els } = await ouvre([LULU], {
    composer: async () => {
      await new Promise((r) => setTimeout(r, 800));
      return COMPOSITION;
    },
  }, { couverture: {} });

  await faireComposer(els);
  assert.strictEqual(els.get('piedDos').textContent, '· composition…');
  assert.strictEqual(els.get('piedMesure').textContent, '',
    'des chiffres sous une composition en cours');

  await new Promise((r) => setTimeout(r, 1000));
  assert.strictEqual(els.get('piedDos').textContent, '· dos 16,5 mm');
});

/**
 * Une composition dure des secondes ; ce qu'on modifie pendant qu'elle tourne rend son
 * résultat faux à l'instant où il arrive. Deux exigences, et la seconde est celle qui
 * fait mal : n'en lancer qu'une à la fois — deux en parallèle se sérialiseraient sur le
 * verrou du Rust et on paierait les deux —, et **recommencer** quand quelque chose a
 * bougé entre-temps, alors même que la composition qui vient de finir a déposé une
 * mesure d'apparence fraîche.
 */
test('une modification pendant la composition la fait recommencer, une fois', async () => {
  let enCours = 0;
  let parallele = 0;
  const { els, appels } = await ouvre([LULU], {
    composer: async () => {
      enCours += 1;
      parallele = Math.max(parallele, enCours);
      await new Promise((r) => setTimeout(r, 1000));
      enCours -= 1;
      return COMPOSITION;
    },
  }, { couverture: {} });
  await faireComposer(els);
  // Le geste ne rend plus la main quand la composition est finie mais quand elle est
  // lancée — c'est tout le propos du chantier. Ce test-là compte les compositions à la
  // milliseconde : il doit voir la première aboutir avant de bousculer la suivante.
  await new Promise((r) => setTimeout(r, 1200));

  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  await new Promise((r) => setTimeout(r, 600));
  // La recomposition tourne depuis 200 ms : la dédicace arrive en plein milieu.
  els.get('inDedicace').value = 'À M.';
  await els.get('inDedicace').declenche('change');
  await new Promise((r) => setTimeout(r, 3000));

  assert.strictEqual(parallele, 1, 'deux compositions en parallèle');
  assert.strictEqual(combien(appels, 'composer'), 3,
    'la modification arrivée en cours de route n\'a pas fait recommencer');
});

/**
 * Le bouton reste un recours, et l'employer doit désarmer la veille : sans quoi une
 * impatience — modifier puis cliquer aussitôt — se paierait d'une seconde composition,
 * qui recalculerait à l'identique ce que le clic venait d'obtenir.
 */
test('composer à la main pendant l\'attente annule la recomposition', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await faireComposer(els);

  els.get('inPoliceInterieur').value = 'Cardo';
  await els.get('inPoliceInterieur').declenche('change');
  await faireComposer(els);
  await attendreComposition();

  assert.strictEqual(combien(appels, 'composer'), 2, 'la veille a recomposé par-dessus');
});

/**
 * Un livre enregistré dans un état périmé réclame bien une composition, mais l'ouvrir
 * n'est pas la demander : on ouvre aussi un `.ozalid` pour regarder sa couverture, et
 * une minute de Typst au premier double-clic serait exactement le genre de zèle qu'on
 * reproche à une application.
 */
test('ouvrir un livre dont la mesure est périmée ne compose rien', async () => {
  const { appels } = await ouvre([LULU], { composer: COMPOSITION }, {
    couverture: {}, dejaCompose: true,
  });
  await attendreComposition();

  assert.strictEqual(combien(appels, 'composer'), 0, 'composé à la seule ouverture');
});

/**
 * La cause qu'aucune estampille ne voyait : le livre lui-même compose des pages
 * liminaires. Une dédicace prend une belle page et son verso blanc — deux pages de plus,
 * et le corps s'ouvre en page 7 au lieu de 5 (`interieur.rs`, test
 * `une_dedicace_ajoute_une_belle_page_et_sa_blanche`). Le gabarit, le papier et la
 * police n'ont pas bougé d'un pouce, et le dos n'est pourtant plus le même.
 *
 * La péremption est volontairement grossière — n'importe quelle modification du livre,
 * sans regarder si elle pagine — pour la même raison que le manuscrit : la liste des
 * champs qui composent vit dans `interieur::source`, et une liste tenue en double ici
 * finirait par diverger sans que rien ne le dise.
 */
test('un dos calculé avant la dédicace ne vaut plus rien', async () => {
  const { els, appels } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernier(appels, 'couverture_apercu')[1].dosMm, 16.513);

  els.get('inDedicace').value = 'À M., qui a tenu la lampe.';
  await els.get('inDedicace').declenche('change');
  await attendreApercu();
  assert.strictEqual(
    dernier(appels, 'couverture_apercu')[1].dosMm,
    null,
    'dos d\'avant la dédicace réutilisé'
  );
});

/**
 * La dernière cause, et la seule qui ne se lise nulle part : le texte fait la
 * pagination. Un dos calculé sur le manuscrit d'avant ne vaut rien même si le gabarit,
 * le papier et la police n'ont pas bougé — c'est précisément ce qui la rend facile à
 * oublier. Les deux portes par lesquelles le texte est remplacé sont exercées ici.
 *
 * Ce qui se regarde a changé avec le chantier « intérieur sans onglet ». Charger un
 * manuscrit périme le dos **et déclenche la composition qui le rétablit** — dans le
 * faux, instantanément, si bien que la planche sans dos n'est jamais demandée : l'aperçu
 * est débouncé, et la seule demande qui parte porte déjà le dos neuf.
 *
 * On fait donc échouer la composition déclenchée. Ce qui reste alors à l'écran est
 * exactement ce que le front fait d'un dos périmé quand rien ne vient le rétablir — et
 * c'est la question que ce test pose depuis le début.
 */
test('un dos calculé sur un autre manuscrit ne vaut plus rien', async () => {
  let compose = true;
  const { els, appels } = await ouvre([LULU], {
    composer: () => {
      if (!compose) throw 'Typst indisponible';
      return COMPOSITION;
    },
  }, { couverture: {} });
  const dernierDos = () => dernier(appels, 'couverture_apercu')[1].dosMm;

  await faireComposer(els);
  await face(els, 'Planche').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernierDos(), 16.513);

  compose = false;
  await faireComposer(els);
  await attendreApercu();
  assert.strictEqual(dernierDos(), null, 'dos gardé après une réimportation du manuscrit');

  await els.get('btChoisirManuscrit').declenche('click');
  await attendreApercu();
  assert.strictEqual(dernierDos(), null, 'dos gardé après un changement de manuscrit');
});

/**
 * Le dos n'est pas seul à sortir du texte, et il est le seul qui ne se lise nulle part :
 * la pagination, les chemins des fichiers composés et les envois déjà écrits en parlent
 * aussi, sous les yeux et en chiffres. Une application dont l'objet est que le nombre de
 * pages soit vrai ne peut pas afficher celui d'un manuscrit qu'on vient de remplacer.
 */
test('réimporter le manuscrit efface ce que l\'ancien texte avait fait afficher', async () => {
  const { els } = await ouvre([LULU], {
    composer: COMPOSITION,
    packager: [{ cle: 'lulu-108x175-broche-standard', libelle: 'Lulu', package: paquet(), vignette: null, erreur: null }],
    epreuve_tirer: '/livres/LHC/epreuve.pdf',
  }, { couverture: {} });

  await faireComposer(els);
  await els.get('btPackager').declenche('click');
  await els.get('btEpreuve').declenche('click');
  // Un envoi porte lui aussi un compte de pages et un dos ; le composer demanderait une
  // liste de dédicataires que ce projet-là n'a pas, et c'est ce qu'il laisse qui compte.
  els.get('resultatEnvois').textContent = 'Rex — envois/rex/ — 262 pages, dos 16,51 mm';
  els.get('resultatEnvois').hidden = false;
  assert.strictEqual(els.get('packages').hidden, false, 'rien à effacer, test sans objet');

  await els.get('btReimporter').declenche('click');

  // La pagination, elle, n'est plus à l'écran mais dans le projet : c'est le Rust qui
  // l'efface au geste qui l'a rendue fausse, et le pied dit alors « dos périmé ». Ce
  // test ne garde que les canaux qui appartiennent en propre à l'écran.
  assert.strictEqual(els.get('packages').hidden, true,
    'les packages de l\'ancien texte restent à lire');
  assert.strictEqual(els.get('resultatEnvois').hidden, true,
    'les envois de l\'ancien texte restent à lire');
  assert.strictEqual(els.get('cheminEpreuve').textContent, '',
    'l\'épreuve de l\'ancien texte reste désignée');
});

/**
 * Remplacer le texte n'est pas changer de livre : le projet, ses destinataires et
 * l'étape où l'on travaille sont les mêmes avant et après. Oublier les sorties du
 * précédent — celles qui renvoient à l'accueil et vident la liste — renverrait au Livre
 * quelqu'un qui venait de réimporter depuis la Livraison.
 */
test('réimporter le manuscrit ne quitte pas l\'étape où l\'on travaille', async () => {
  const { els } = await ouvre([LULU], { composer: COMPOSITION }, { couverture: {} });
  await els.get('onglet-livraison').declenche('click');
  assert.strictEqual(els.get('etapeLivraison').hidden, false);

  await els.get('btReimporter').declenche('click');

  assert.strictEqual(els.get('etapeLivraison').hidden, false,
    'un réimport a renvoyé au Livre');
  assert.ok(els.get('destinataires').textes('span').includes('Lulu — poche 108 × 175'),
    'un réimport a vidé la liste des destinataires');
});
