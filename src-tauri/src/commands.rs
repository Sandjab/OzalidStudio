//! Commandes exposées à l'interface. Aucune logique métier ici : elles orchestrent
//! les modules, tiennent le projet ouvert et traduisent les erreurs en messages
//! affichables.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;
use tauri::Manager;
use tauri::State;

use crate::catalogue::{self, Provider};
use crate::couverture::{self, Couverture, Ressource};
use crate::ebook;
use crate::epreuve;
use crate::import;
use crate::interieur::{self, Interieur, Reglage};
use crate::manuscrit;
use crate::maquettes;
use crate::package;
use crate::planche;
use crate::preferences;
use crate::projet::{Livrable, Livraison, Livre, Mesure, Projet};
use crate::typst::Typst;

/// Les fichiers de catalogue du poste que le démarrage a refusés. Vide sur un poste qui
/// n'en dépose aucun, c'est-à-dire presque toujours.
pub struct CatalogueRefus(pub Vec<crate::catalogue::Refus>);

/// Le projet ouvert. Un seul à la fois : c'est un éditeur de document, pas une
/// bibliothèque. `chemin` est absent tant que le projet n'a pas été enregistré.
#[derive(Default)]
pub struct Atelier {
    ouvert: Mutex<Option<Ouvert>>,
}

struct Ouvert {
    chemin: Option<PathBuf>,
    projet: Projet,
    /// Vrai dès qu'une commande a touché au projet sans qu'il ait été réécrit.
    /// C'est lui, et lui seul, qui décide si fermer perd du travail.
    modifie: bool,
    /// La dernière image générée pour un envoi, tant qu'elle n'a pas été acceptée.
    ///
    /// Elle vit **hors du projet** : un modèle de diffusion rend rarement une écriture
    /// lisible du premier coup, et l'archive n'a pas à conserver la suite des essais.
    /// Accepter la fait entrer dans le `.ozalid` ; fermer le projet la laisse là où elle
    /// était, c'est-à-dire nulle part.
    candidat: Option<(usize, Vec<u8>)>,
}

/// Vue d'un gabarit d'intérieur pour l'interface : POD × format × reliure.
///
/// Le papier n'y a pas sa place, et ce n'est pas un oubli : il appartient à l'identité
/// du livrable, jamais du gabarit, et deux livrables qui ne diffèrent que par lui
/// partagent cette même entrée. Qui veut l'offre complète d'un POD, papiers compris, la
/// lit dans l'arbre — `PodVue`, servie par `pods_liste`.
#[derive(Serialize)]
pub struct ProviderVue {
    cle: String,
    /// Les trois axes du gabarit, tels que la fabrication d'office de cette entrée les
    /// porte. L'écran en a besoin pour composer la `Fabrication` qu'il envoie à
    /// `livrable_ajouter` : la clé se fabrique et se compare, elle ne se découpe jamais.
    pod: String,
    format: String,
    reliure: String,
    libelle: String,
    largeur: f64,
    hauteur: f64,
    fond_perdu: Option<f64>,
}

#[derive(Serialize)]
pub struct PapierVue {
    cle: String,
    libelle: String,
    /// La couleur du papier, telle que le canevas des envois la peint. Elle traverse
    /// jusqu'ici parce que c'est l'écran qui s'en sert, jamais la composition.
    teinte: String,
    /// Vrai quand **ce papier** publie de quoi calculer le dos. Faux, la ligne réclame
    /// un relevé plutôt que de laisser croire à un chiffre. Porté par le papier et non
    /// par le POD : un POD peut publier une formule pour l'un et pas pour l'autre, et
    /// c'est le papier retenu qui décide.
    dos_publie: bool,
}

impl From<&catalogue::Papier> for PapierVue {
    fn from(pa: &catalogue::Papier) -> Self {
        Self {
            cle: pa.cle.clone(),
            libelle: pa.nom.clone(),
            teinte: pa.teinte.clone(),
            dos_publie: pa.dos.publie(),
        }
    }
}

impl From<&Provider> for ProviderVue {
    fn from(p: &Provider) -> Self {
        Self {
            cle: p.cle.clone(),
            pod: p.fabrication.pod.clone(),
            format: p.fabrication.format.clone(),
            reliure: p.fabrication.reliure.clone(),
            libelle: p.libelle.clone(),
            largeur: p.format.0,
            hauteur: p.format.1,
            fond_perdu: p.fond_perdu,
        }
    }
}

/// Ce qu'un POD offre, en arbre : la cascade de l'ajout y lit ses formats, les trois
/// réglages de la ligne y lisent ses reliures, ses finitions et ses papiers.
///
/// Distincte de `ProviderVue`, et non un champ de plus sur elle : celle-là est une
/// projection POD × format, qui n'a pas de place pour dire ce qu'un POD offre d'autre.
/// Les deux cohabitent — la plate pour ce que la projection sait seule dire (format en
/// mm, fond perdu effectif, libellé composé), l'arbre pour les choix.
#[derive(Serialize)]
pub struct PodVue {
    cle: String,
    nom: String,
    formats: Vec<FormatVue>,
    reliures: Vec<ReliureVue>,
    finitions: Vec<FinitionVue>,
    papiers: Vec<PapierVue>,
}

#[derive(Serialize)]
pub struct FormatVue {
    cle: String,
    nom: String,
}

impl From<&catalogue::Format> for FormatVue {
    fn from(f: &catalogue::Format) -> Self {
        Self {
            cle: f.cle.clone(),
            nom: f.nom.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct ReliureVue {
    cle: String,
    nom: String,
    /// Pourquoi on ne la compose pas, telle que le fichier l'écrit — `null` chez une
    /// reliure composable. C'est le fichier qui tranche : `verifie_reliure` refuse une
    /// reliure qui porterait à la fois une géométrie et une raison de ne pas en avoir,
    /// donc l'écran n'a pas à interroger la géométrie pour savoir quoi griser.
    non_outille: Option<String>,
}

impl From<&catalogue::Reliure> for ReliureVue {
    fn from(r: &catalogue::Reliure) -> Self {
        Self {
            cle: r.cle.clone(),
            nom: r.nom.clone(),
            non_outille: r.non_outille.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct FinitionVue {
    cle: String,
    nom: String,
}

impl From<&catalogue::Finition> for FinitionVue {
    fn from(f: &catalogue::Finition) -> Self {
        Self {
            cle: f.cle.clone(),
            nom: f.nom.clone(),
        }
    }
}

impl From<&catalogue::Pod> for PodVue {
    fn from(pod: &catalogue::Pod) -> Self {
        Self {
            cle: pod.cle.clone(),
            nom: pod.nom.clone(),
            formats: pod.formats.iter().map(FormatVue::from).collect(),
            reliures: pod.reliures.iter().map(ReliureVue::from).collect(),
            finitions: pod.finitions.iter().map(FinitionVue::from).collect(),
            papiers: pod.papiers.iter().map(PapierVue::from).collect(),
        }
    }
}

/// Ce que l'interface affiche d'un projet ouvert.
#[derive(Serialize)]
pub struct ProjetVue {
    pub chemin: Option<String>,
    pub livre: Livre,
    pub manuscrit_source: Option<String>,
    /// Chapitres réellement trouvés dans le manuscrit embarqué.
    pub chapitres_trouves: u32,
    pub mots: u32,
    /// Vrai quand le projet ne porte aucun texte. Distinct de « zéro chapitre » :
    /// un manuscrit présent mais non composable en trouve zéro aussi, et ce n'est
    /// pas la même chose à corriger.
    pub manuscrit_absent: bool,
    /// Modifications non enregistrées.
    pub modifie: bool,
    /// Maquette de couverture du projet, si le projet en porte une.
    pub couverture: Option<Couverture>,
    pub couverture_importee: bool,
    pub images: Vec<String>,
    pub interieur: Interieur,
    /// Le PDF de l'intérieur composé pour le livrable visé, s'il est sur le disque.
    ///
    /// **Dérivé, jamais retenu.** Un `.ozalid` déplacé — ou ouvert sur une autre machine,
    /// ce pour quoi il est fait — porterait un chemin absolu qui ne mène nulle part. Il
    /// se recalcule à chaque vue, et l'existence du fichier est vérifiée : un lien vers
    /// un PDF effacé à la main est pire que pas de lien.
    ///
    /// Absent tant que la mesure l'est. Un PDF qui traîne d'une composition périmée n'est
    /// pas celui du livre qu'on regarde, et le montrer ferait relire une pagination que
    /// le pied vient de déclarer fausse.
    pub interieur_pdf: Option<String>,
    /// Les livrables du livre et celui qu'on vise. Chacun porte son identité à quatre
    /// axes et la clé de son gabarit : c'est par celle-là que le front joint la table
    /// des gabarits — les libellés, les formats et les papiers viennent de là. Une
    /// **vue** depuis la v5 : `compose` y est recalculée par livrable, la donnée range
    /// la mesure sous le gabarit.
    pub livraison: LivraisonVue,
    /// Les livrables que l'ouverture a retirés faute de catalogue qui les porte encore.
    /// Vide partout ailleurs — un projet neuf n'a rien perdu — et l'écran se tait alors,
    /// comme il se tait sur les fichiers de catalogue refusés quand il n'y en a pas.
    pub elagues: Vec<String>,
    /// La main du livre et ses envois. Toujours sérialisée, même vide : le front y
    /// lit la liste sans avoir à se demander si la section existe.
    pub envois: crate::envoi::Envois,
}

#[derive(Serialize)]
pub struct Composition {
    /// Le projet tel qu'il ressort de la composition : c'est lui qui porte désormais la
    /// mesure, rangée sous le gabarit du livrable visé. Les quatre chiffres ci-dessous
    /// en sont une copie de lecture, issue du même calcul — le compte rendu de l'écran
    /// les lit sans avoir à retrouver le livrable.
    pub projet: ProjetVue,
    pub pages: u32,
    pub gouttiere: f64,
    pub blanche: bool,
    pub chapitres: u32,
    /// Épaisseur du dos en mm, ou `null` chez un imprimeur à gabarit. C'est cette
    /// valeur qui alimentera la planche : elle n'est jamais ressaisie.
    pub dos: Option<f64>,
    pub pdf: String,
    /// Familles que Typst n'a pas trouvées et a remplacées par une écriture de repli
    /// — sans échouer, donc sans que rien d'autre ne le dise. Vide, tout va bien.
    pub polices_introuvables: Vec<String>,
}

#[tauri::command]
pub fn providers_liste() -> Vec<ProviderVue> {
    catalogue::providers()
        .iter()
        .map(ProviderVue::from)
        .collect()
}

/// L'arbre du catalogue : un POD, ses formats, ses reliures, ses finitions, ses papiers.
///
/// Pas de filtre ici : `Pod::verifie` refuse au chargement un POD dont aucune reliure ne
/// porte de géométrie, en nommant son fichier ; filtrer ici escamoterait l'imprimeur au
/// lieu de le signaler.
#[tauri::command]
pub fn pods_liste() -> Vec<PodVue> {
    catalogue::pods().iter().map(PodVue::from).collect()
}

/// Ce que le démarrage a refusé de charger. L'interface le dit à la Livraison : c'est là
/// qu'on regarde la liste des POD, donc là qu'un POD manquant se remarque.
#[tauri::command]
pub fn catalogue_refus(refus: State<CatalogueRefus>) -> Vec<catalogue::Refus> {
    refus.0.clone()
}

/// Importe un répertoire de travail de l'ancienne chaîne (son `livre.toml`).
/// Le projet devient le projet ouvert, sans être enregistré : l'utilisateur choisit
/// où poser le `.ozalid`.
#[tauri::command]
pub fn projet_importer(livre_toml: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let projet = import::depuis_livre_toml(Path::new(&livre_toml))?;
    poser(&atelier, None, projet, true)
}

/// Un projet vide, à remplir.
///
/// Ni assistant ni sélecteur de fichiers : c'est un document neuf, comme dans un
/// traitement de texte. Le manuscrit se choisit quand on veut, l'enregistrement se
/// fait quand on veut. Le projet n'est pas « modifié » : il n'y a encore rien à
/// perdre, et le premier champ saisi lèvera le drapeau.
#[tauri::command]
pub fn projet_nouveau(atelier: State<Atelier>, app: tauri::AppHandle) -> Result<ProjetVue, String> {
    poser(&atelier, None, projet_neuf(gabarit_de_depart(&app)), false)
}

/// Le projet neuf que sert `projet_nouveau`, gabarit de départ compris.
///
/// À part de la commande pour être vérifiable : une commande Tauri réclame un `State` et
/// un `AppHandle` qu'aucun test ne fabrique, et la ligne qui pose le gabarit serait alors
/// la seule du chantier que rien ne protège.
fn projet_neuf(gabarit: String) -> Projet {
    let mut p = Projet::nouveau(Livre::vide(), String::new());
    p.meta.envois.gabarit = gabarit;
    p
}

/// Le gabarit de départ tel que les préférences le portent, celui de la maison à défaut.
///
/// Un répertoire de configuration introuvable ne fait pas échouer la création d'un
/// projet : on part alors du gabarit de la maison, ce qui est exactement l'état d'un
/// poste où rien n'a encore été réglé.
fn gabarit_de_depart(app: &tauri::AppHandle) -> String {
    config(app)
        .map(|d| preferences::charger(&d).gabarit_defaut)
        .unwrap_or_else(|| crate::diffusion::GABARIT_DEFAUT.into())
}

/// Referme le projet sans rien écrire.
///
/// La garde des modifications appartient à l'appelant : cette commande ne demande
/// rien, elle exécute. Les séparer permet à l'interface de poser la même question
/// avant Nouveau, Ouvrir, Importer et la fermeture de la fenêtre.
#[tauri::command]
pub fn projet_fermer(atelier: State<Atelier>) {
    *atelier.ouvert.lock().unwrap() = None;
}

/// Réécrit le projet là où il a déjà été enregistré.
///
/// Sans chemin mémorisé, l'interface bascule sur « Enregistrer sous… » : elle seule
/// possède le sélecteur de fichiers.
#[tauri::command]
pub fn projet_enregistrer(
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let (vue, chemin) = {
        let mut garde = atelier.ouvert.lock().unwrap();
        let o = garde.as_mut().ok_or_else(aucun_projet)?;
        let chemin = o
            .chemin
            .clone()
            .ok_or_else(|| "projet jamais enregistré : choisir où le poser.".to_string())?;
        (enregistrer_a(o, &chemin)?, chemin)
    };
    memoriser(&app, &chemin);
    Ok(vue)
}

#[tauri::command]
pub fn projet_ouvrir(
    chemin: String,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let c = PathBuf::from(&chemin);
    let projet = Projet::ouvrir(&c)?;
    let vue = poser(&atelier, Some(c.clone()), projet, false)?;
    memoriser(&app, &c);
    Ok(vue)
}

#[tauri::command]
pub fn projet_enregistrer_sous(
    chemin: String,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let c = PathBuf::from(&chemin);
    let vue = {
        let mut garde = atelier.ouvert.lock().unwrap();
        let o = garde.as_mut().ok_or_else(aucun_projet)?;
        enregistrer_a(o, &c)?
    };
    memoriser(&app, &c);
    Ok(vue)
}

/// Les projets récents dont le fichier existe encore.
///
/// L'écran d'accueil et le sous-menu « Ouvrir un récent » lisent cette même liste :
/// il n'y a pas deux inventaires à tenir d'accord.
#[tauri::command]
pub fn recents_liste(app: tauri::AppHandle) -> Vec<String> {
    config(&app)
        .map(|d| preferences::charger(&d).recents_existants())
        .unwrap_or_default()
}

/// Libellés des trois boutons de la garde.
///
/// Ce sont eux qui font foi au retour : avec une variante personnalisée, le plugin
/// rend `MessageDialogResult::Custom(libellé)` et non un `Yes`/`No`. Les garder en
/// constantes évite que la comparaison et l'affichage divergent.
const ENREGISTRER: &str = "Enregistrer";
const IGNORER: &str = "Ne pas enregistrer";
const ANNULER: &str = "Annuler";

/// Demande quoi faire des modifications non enregistrées.
///
/// Rend `"enregistrer"`, `"ignorer"` ou `"annuler"`, et `"ignorer"` d'emblée quand
/// il n'y a rien à perdre. La commande **ne fait rien** de la réponse : c'est
/// l'interface qui agit, parce qu'elle seule possède le sélecteur de fichiers dont
/// « Enregistrer sous… » a besoin.
///
/// `async` par nécessité : `blocking_show_with_result` bloque son fil jusqu'au clic,
/// et le plugin interdit de l'appeler depuis le fil principal — ce qui serait le cas
/// d'une commande synchrone, dont le corps s'exécute en ligne dans le gestionnaire
/// de protocole de la webview.
#[tauri::command]
pub async fn garde_modifications(
    app: tauri::AppHandle,
    atelier: State<'_, Atelier>,
) -> Result<String, String> {
    // Le verrou est relâché avant la boîte : la tenir pendant que l'utilisateur
    // réfléchit condamnerait toute autre commande.
    let modifie = {
        let garde = atelier.ouvert.lock().unwrap();
        garde.as_ref().is_some_and(|o| o.modifie)
    };
    if !modifie {
        return Ok("ignorer".into());
    }

    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    let reponse = app
        .dialog()
        .message("Ce projet porte des modifications qui ne sont pas enregistrées.")
        .title("Enregistrer avant de continuer ?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::YesNoCancelCustom(
            ENREGISTRER.into(),
            IGNORER.into(),
            ANNULER.into(),
        ))
        .blocking_show_with_result();

    Ok(reponse_garde(reponse).to_string())
}

/// Ce que le clic de l'utilisateur veut dire.
///
/// Séparé de la boîte parce que la boîte ne se simule pas, alors que cette
/// traduction, elle, se teste — et qu'une erreur ici perdrait du travail.
fn reponse_garde(r: tauri_plugin_dialog::MessageDialogResult) -> &'static str {
    use tauri_plugin_dialog::MessageDialogResult;
    match r {
        MessageDialogResult::Custom(s) if s == ENREGISTRER => "enregistrer",
        MessageDialogResult::Custom(s) if s == IGNORER => "ignorer",
        // Filet : si une plateforme rendait les valeurs canoniques plutôt que les
        // libellés, le sens resterait le même. Tout le reste — fermeture de la
        // boîte comprise — est un refus, parce que c'est le choix qui ne perd rien.
        MessageDialogResult::Yes => "enregistrer",
        MessageDialogResult::No => "ignorer",
        _ => "annuler",
    }
}

/// L'interface a-t-elle posé ses écouteurs ?
///
/// Tant qu'elle ne l'a pas fait, retenir la fermeture rendrait l'application
/// inquittable : personne n'écouterait la demande. Un front qui n'a jamais démarré
/// n'a rien à perdre non plus — on le laisse donc partir sans question.
///
/// Ce que ce filet suppose, et qui le rend sûr : le seul chemin vers
/// `modifie = true` passe par `vue_modifiee`, elle-même appelée uniquement par des
/// commandes qui exigent un projet déjà ouvert — et un projet ne s'ouvre que par une
/// commande du front. `Atelier` naît vide (`Default`), donc tant que l'interface n'a
/// pas tourné, il n'y a rien à perdre. **Si un jour `setup()` restaure ou reprend un
/// projet automatiquement, cet invariant casse** : le filet laisserait alors partir
/// un projet modifié sans le demander.
#[derive(Default)]
pub struct Interface {
    pub prete: std::sync::atomic::AtomicBool,
}

/// L'interface annonce qu'elle écoute. Appelée une fois, au chargement.
#[tauri::command]
pub fn interface_prete(interface: State<Interface>) {
    // `Relaxed` suffit : ce drapeau ne publie rien d'autre que lui-même — l'état
    // partagé qui compte, `Atelier.ouvert`, a son propre `Mutex`. Les lecteurs ne
    // font que décider d'émettre ou non, jamais lire une valeur posée à côté.
    interface
        .prete
        .store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Relit le manuscrit à sa source d'origine et remplace la copie embarquée.
///
/// Le `.ozalid` est auto-portant : le manuscrit y est copié, donc une correction faite
/// dans l'éditeur de texte n'y entre que par ce geste. Le chemin d'origine est
/// mémorisé pour que ce soit un bouton et non une navigation.
#[tauri::command]
pub fn manuscrit_reimporter(atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let source = o.projet.meta.manuscrit.source.clone().ok_or_else(|| {
        "ce projet ne mémorise aucune source de manuscrit — en choisir une.".to_string()
    })?;
    let texte = std::fs::read_to_string(&source)
        .map_err(|e| format!("manuscrit introuvable ({source}) : {e}"))?;
    o.projet.remplacer_texte(texte);
    vue_modifiee(o)
}

/// Remplace le manuscrit par un fichier choisi, et mémorise son chemin.
#[tauri::command]
pub fn manuscrit_choisir(chemin: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let texte =
        std::fs::read_to_string(&chemin).map_err(|e| format!("manuscrit illisible : {e}"))?;
    o.projet.remplacer_texte(texte);
    o.projet.meta.manuscrit.source = Some(chemin);
    vue_modifiee(o)
}

#[tauri::command]
pub fn livre_modifier(livre: Livre, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.modifier_livre(livre);
    vue_modifiee(o)
}

/// Les jetons que les champs libres du livre peuvent citer : les clés du livre, et
/// l'imprimeur du livrable.
#[tauri::command]
pub fn jetons_liste() -> Vec<&'static str> {
    crate::gabarit::jetons()
}

#[tauri::command]
pub fn polices_texte_liste() -> Vec<&'static str> {
    interieur::POLICES_TEXTE.to_vec()
}

/// L'écriture d'intérieur choisie, en donnée `data:`, pour l'échantillon de l'onglet
/// Livre. Comme les aperçus : la fenêtre ne lit pas les fichiers, une police n'y entre
/// pas autrement.
///
/// Les octets sont ceux que Typst composera, pris dans les mêmes répertoires. Un
/// échantillon rendu dans la police du poste montrerait une écriture que le livre
/// n'aura pas — et c'est un mensonge qu'aucune fenêtre ne rattrape : le repli d'un
/// navigateur est muet, comme celui de Typst.
///
/// Le romain seul : l'échantillon montre une écriture, pas ses coupes. La lecture
/// parcourt les répertoires de polices en entier — c'est le prix de `polices_du_livre`,
/// et il se paie une fois par famille, la fenêtre gardant ce qu'elle a reçu.
#[tauri::command]
pub fn police_texte_donnee(famille: String) -> Result<String, String> {
    let typst = typst()?;
    let polices = crate::ebook::polices_du_livre(&famille, typst.polices()).ok_or_else(|| {
        format!("police d'intérieur « {famille} » introuvable dans les polices embarquées")
    })?;
    Ok(format!(
        "data:font/ttf;base64,{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            &polices.romain.octets
        )
    ))
}

#[tauri::command]
pub fn interieur_modifier(
    interieur: Interieur,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    interieur.verifie()?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.modifier_interieur(interieur);
    vue_modifiee(o)
}

/* ---------- livrables ---------- */

/// Le livrable visé, résolu : sa vue plate, son papier, et le livrable lui-même.
///
/// Le point de passage unique de tout ce qui a besoin d'un gabarit : composer,
/// apercevoir, mesurer un dos. Il n'y a plus de second endroit où le choisir.
fn vise(o: &Ouvert) -> Result<(Provider, catalogue::Papier, &Livrable), String> {
    let l = o
        .projet
        .meta
        .livraison
        .courant()
        .ok_or("aucun livrable : en déclarer un à l'étape Livraison.")?;
    let r = catalogue::resout(&l.fabrication)?;
    Ok((r.provider(), r.papier.clone(), l))
}

/// La cible de packaging d'un livrable : ce que `package` réclame pour composer.
///
/// Fonction libre et non deux montages recopiés : `packager` et `envoyer` la bâtissent
/// tous deux, et la seule chose qui distingue leurs six champs — un papier avec la clé
/// d'un autre livrable — écrirait un dos faux dans le bon répertoire.
fn cible(pr: Provider, papier: catalogue::Papier, d: &Livrable) -> package::Cible {
    package::Cible {
        pr,
        papier,
        releve: planche::Releve {
            dos: d.dos_mm,
            fond_perdu: d.fond_perdu_mm,
        },
        // Le nom d'imprimeur, pas la clé : c'est lui que la fiche de téléversement
        // porte, et « mat » ne se coche sur aucun bon de commande.
        finition: nom_finition(d, catalogue::pod(&d.fabrication.pod)),
        cle: d.cle(),
    }
}

/// Le refus d'un livrable déjà déclaré, à quatre axes — la finition n'y est pas.
///
/// Fonction libre plutôt qu'une ligne dans la commande : une commande réclame un `State`
/// qu'aucun test ne fabrique, et la règle qui décide ce qu'est « le même livrable »
/// serait alors la seule du chantier que rien ne protège.
fn refuse_doublon(livrables: &[Livrable], cle: &str) -> bool {
    livrables.iter().any(|x| x.cle() == cle)
}

/// Ce qui interdit de régler cette ligne, s'il y a lieu.
///
/// Hors de la commande pour la même raison que `refuse_doublon` : une commande réclame
/// un `State` qu'aucun test ne fabrique, et ces deux refus-là seraient alors les seuls
/// du chantier que rien ne protège.
///
/// Le POD et le format ne se règlent pas : ils se choisissent à l'ajout, en cascade, et
/// les changer sur place laisserait le livrable sous une pagination qui n'est plus la
/// sienne — retirer puis ajouter le dit, et le fait. La reliure, elle, se règle (spec
/// § 6) : elle emporte le gabarit avec elle, le livrable retombe sur un gabarit sans
/// mesure, et la recomposition est précisément ce qu'elle exige. La finition doit
/// exister chez le POD : elle nomme une option de commande, et une option inventée ne se
/// commande nulle part.
fn reglage_refuse(place: &Livrable, neuf: &Livrable, pod: &catalogue::Pod) -> Option<String> {
    if place.fabrication.pod != neuf.fabrication.pod
        || place.fabrication.format != neuf.fabrication.format
    {
        return Some(
            "le POD et le format d'un livrable ne se règlent pas : retirer, puis ajouter.".into(),
        );
    }
    match &neuf.finition {
        Some(f) if !pod.finitions.iter().any(|x| &x.cle == f) => {
            Some(format!("finition inconnue chez {} : {f}.", pod.nom))
        }
        _ => None,
    }
}

/// Ajoute un livrable au livre.
///
/// Le refus du doublon porte sur les **quatre axes de fabrication** : deux livrables qui
/// ne différeraient que par la finition produiraient les mêmes octets dans deux
/// répertoires (spec § 4) — la finition est une donnée de commande, pas de fabrication.
#[tauri::command]
pub fn livrable_ajouter(
    fabrication: catalogue::Fabrication,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let r = catalogue::resout(&fabrication)?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    if refuse_doublon(&l.livrables, &fabrication.cle()) {
        return Err(format!(
            "{} en {} est déjà un livrable de ce livre — la finition seule n'en fait \
             pas un autre : le fichier produit serait le même.",
            r.pod.nom, r.papier.nom
        ));
    }
    l.livrables.push(Livrable::pour(fabrication));
    vue_modifiee(o)
}

/// Retire un livrable — sauf le dernier : c'est lui qui donne son format à
/// l'aperçu, et une liste vide rendrait la Couverture inutilisable.
#[tauri::command]
pub fn livrable_retirer(cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    if l.livrables.len() < 2 {
        return Err(
            "un livre garde au moins un livrable : c'est lui qui donne le format \
             sous lequel on regarde la couverture."
                .into(),
        );
    }
    let avant = l.livrables.len();
    l.livrables.retain(|d| d.cle() != cle);
    if l.livrables.len() == avant {
        return Err(format!("{cle} n'est pas un livrable de ce livre."));
    }
    // Retirer celui qu'on visait laisse le pointeur en l'air : il retombe sur le
    // premier, plutôt que de désigner un absent jusqu'au prochain geste.
    if l.courant().is_none() {
        l.courant = l.livrables[0].cle();
    }
    vue_modifiee(o)
}

/// La reliure, le papier, la finition et les relevés d'un livrable. `cle` désigne le
/// livrable tel qu'il était : changer sa reliure ou son papier change son identité, et
/// `courant` suit.
#[tauri::command]
pub fn livrable_regler(
    cle: String,
    livrable: Livrable,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    // Le candidat est résolu **avant** d'être posé : un axe ou un papier inconnu doit
    // laisser le livrable tel qu'il était, et non l'abandonner à moitié réglé.
    let r = catalogue::resout(&livrable.fabrication)?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    let neuve = livrable.cle();
    // La ligne visée se trouve **avant** qu'on regarde le doublon : une `cle` que le
    // livre ne porte pas n'est pas un doublon, et répondre « déjà un livrable » se
    // lirait comme un refus de ce qu'on croyait régler. Le rang sert dans la foulée,
    // sans que rien ne bouge entre-temps : ce n'est pas un pointeur retenu.
    let rang = l
        .livrables
        .iter()
        .position(|x| x.cle() == cle)
        .ok_or_else(|| format!("{cle} n'est pas un livrable de ce livre."))?;
    if neuve != cle && refuse_doublon(&l.livrables, &neuve) {
        return Err(format!("{neuve} est déjà un livrable de ce livre."));
    }
    let place = &mut l.livrables[rang];
    // Refusé avant toute écriture : le POD et le format ne se règlent pas sur une ligne,
    // ils se choisissent à l'ajout. La reliure, elle, emporte le gabarit avec elle — le
    // livrable retombe alors sur un gabarit sans mesure, et recompose, ce qui est
    // précisément ce qu'une reliure exige. Le papier ne touche à rien : deux papiers
    // partagent la mesure de leur gabarit, et chacun en tire son dos à la vue.
    if let Some(e) = reglage_refuse(place, &livrable, r.pod) {
        return Err(e);
    }
    *place = livrable;
    if l.courant == cle {
        l.courant = neuve;
    }
    vue_modifiee(o)
}

/// Déplace le pointeur : pour qui l'on compose, et sous quel format on regarde.
///
/// Le geste modifie le projet, parce que le pointeur est enregistré avec lui : rouvrir
/// un livre le rend tel qu'on l'avait laissé, visé sur le même livrable.
#[tauri::command]
pub fn livrable_viser(cle: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let l = &mut o.projet.meta.livraison;
    if !l.livrables.iter().any(|d| d.cle() == cle) {
        return Err(format!("{cle} n'est pas un livrable de ce livre."));
    }
    l.courant = cle;
    vue_modifiee(o)
}

/// Compose l'intérieur du projet ouvert pour le livrable visé, et rend le compte
/// de pages avec le dos qui en découle.
#[tauri::command]
pub fn composer(atelier: State<Atelier>) -> Result<Composition, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let (pr, papier, livrable) = vise(o)?;
    // La clé de rangement de la mesure et l'empreinte du gabarit qui compose, prises au
    // même endroit — le livrable visé — parce que l'emprunt s'arrête ici : `pr.cle` vaut
    // la clé de gabarit, mais par construction de `Resolu::provider`, pas par contrat.
    // L'empreinte, elle, dira à la réouverture si le fichier de POD a été réécrit
    // entre-temps (spec § 8).
    let gabarit = livrable.fabrication.cle_gabarit();
    let empreinte = catalogue::resout(&livrable.fabrication)?.empreinte();

    let dossier = sorties_dossier(o, &pr.cle)?;
    let typst = typst()?;
    // La même composition que celle du package : convergence, puis PDF, écrits sous la
    // clé du **gabarit**. Deux papiers du même gabarit y composent le même intérieur.
    let r = package::composer_interieur(&o.projet, &pr, &pr.cle, &dossier, &typst)?;
    let polices_introuvables = r.polices_introuvables.clone();

    // Le compte rendu dit « Chapitres » : une préface ou une page de partie n'en est
    // pas un, et l'onglet Livre en affiche déjà le compte juste. Recompté ici, sur le
    // même découpage que celui de la composition — qui, lui, ne le rend pas.
    let chapitres = manuscrit::decoupe(&o.projet.texte, o.projet.meta.livre.chapitres)?
        .iter()
        .filter(|p| p.est_chapitre())
        .count() as u32;
    let dos = papier.dos.mm(r.pages);

    // La mesure entre dans le projet, sous le gabarit pour qui elle a été faite :
    // revenir à ce gabarit, ou rouvrir le livre, ne la fera plus recalculer. Le repli
    // de police y entre avec elle : il décrit le PDF qui vient d'être écrit, et ce PDF
    // ne redevient pas juste en refermant le livre. Le dos, lui, n'y entre pas : il
    // suit le papier, et se recalcule à chaque vue.
    o.projet.meta.livraison.retenir_mesure(
        &gabarit,
        Mesure {
            pages: r.pages,
            gouttiere: r.gouttiere,
            blanche: r.blanche,
            empreinte: Some(empreinte),
            polices_introuvables: polices_introuvables.clone(),
        },
    );

    Ok(Composition {
        projet: vue_modifiee(o)?,
        pages: r.pages,
        gouttiere: r.gouttiere,
        blanche: r.blanche,
        chapitres,
        dos,
        pdf: r.pdf.to_string_lossy().into_owned(),
        polices_introuvables,
    })
}

/// Tire l'épreuve de relecture à la racine des sorties : elle ne vise aucun éditeur,
/// elle ne descend donc pas dans un répertoire de livrable.
#[tauri::command]
pub fn epreuve_tirer(corps_pt: f64, atelier: State<Atelier>) -> Result<String, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let livre = &o.projet.meta.livre;
    let int = &o.projet.meta.interieur;
    // `epreuve::source` interpole la police sans échappement : la validation est ici.
    int.verifie()?;
    let chapitres = manuscrit::decoupe(&o.projet.texte, livre.chapitres)?;

    let dossier = sorties_racine(o)?;
    std::fs::create_dir_all(&dossier).map_err(|e| {
        format!(
            "répertoire de sortie inutilisable ({}) : {e}",
            dossier.display()
        )
    })?;
    let src = dossier.join("epreuve.typ");
    ecrire(&src, &epreuve::source(livre, int, &chapitres, corps_pt))?;
    let pdf = dossier.join("epreuve.pdf");
    // Les substitutions de police ne sont pas remontées ici : l'épreuve se lit pour
    // son texte, et composer l'intérieur — qui emploie les mêmes polices — les
    // signale déjà dans son compte rendu.
    typst()?.compile(&src, &pdf)?;
    Ok(pdf.to_string_lossy().into_owned())
}

/* ---------- couverture ---------- */

#[derive(Serialize)]
pub struct MaquetteVue {
    cle: String,
    libelle: String,
    /// Ni renommable, ni effaçable. La fenêtre s'en sert pour ne pas offrir des gestes
    /// que le Rust refuserait de toute façon — l'interface est une politesse, le refus
    /// est ailleurs.
    fournie: bool,
}

#[tauri::command]
pub fn maquettes_liste(app: tauri::AppHandle) -> Vec<MaquetteVue> {
    maquettes::toutes(config(&app).as_deref())
        .into_iter()
        .map(|m| MaquetteVue {
            cle: m.cle,
            libelle: m.nom,
            fournie: m.fournie,
        })
        .collect()
}

#[tauri::command]
pub fn polices_liste() -> Vec<&'static str> {
    couverture::POLICES.to_vec()
}

/// Charge une maquette de départ. Elle remplace la mise en page, jamais l'identité du
/// livre — le titre et l'auteur imprimés restent ceux du projet —, ni ses photos.
///
/// Les images de la maquette ne font que **combler**, par [`combler_images`] : un livre
/// déjà illustré garde les siennes, et seule une face qu'il laisse nue reçoit celle de
/// l'archive. Une photo appartient au livre, pas à la mise en page : deux fournies en
/// portent une, comme toute personnalisée écrite depuis un projet illustré, et aucune
/// n'a à l'imposer au livre ouvert.
#[tauri::command]
pub fn maquette_choisir(
    cle: String,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let m = maquettes::par_cle(config(&app).as_deref(), &cle)
        .ok_or_else(|| format!("maquette inconnue : {cle}"))?;
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.meta.couverture.maquette = Some(m.couverture);
    combler_images(&mut o.projet.images, &m.images);
    vue_modifiee(o)
}

/// La vignette d'une maquette : sa 1ère de couverture, composée sur le livre ouvert.
///
/// Ce n'est pas l'archive qu'on montre, c'est ce que la **choisir** donnerait — le titre
/// et l'auteur du projet, ses photos là où la maquette n'en porte pas, le format du
/// livrable visé. Une vignette qui montrerait l'archive nue promettrait autre chose que
/// le bouton d'à côté, et c'est [`images_vignette`] qui l'en empêche.
///
/// Le projet n'est pas touché : la fusion se fait sur une copie de sa table d'images.
///
/// `dos_mm` vient de la fenêtre, comme pour [`couverture_apercu`] : il ne sert ici qu'au
/// prolongement panoramique d'une photo qui traverse la planche, et une vignette se
/// compose très bien sans, sur un livre dont l'intérieur n'a pas encore paginé.
///
/// **60 ppi et non les 150 de l'aperçu** : mesuré, la rastérisation ne coûte rien — c'est
/// la composition qui prend ses trente millisecondes —, mais une vignette y pèse 6 Kio
/// contre 20, et elles voyagent toutes ensemble dans la même page, en `data:`.
#[tauri::command]
pub fn maquette_apercu(
    cle: String,
    dos_mm: Option<f64>,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<String, String> {
    let m = maquettes::par_cle(config(&app).as_deref(), &cle)
        .ok_or_else(|| format!("maquette inconnue : {cle}"))?;
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (pr, _, _) = vise(o)?;

    // Un répertoire à part de celui de l'aperçu : les deux se composent en même temps —
    // la fenêtre garde son aperçu affiché derrière le dialogue — et partager le dossier
    // ferait écrire à l'un les images de l'autre.
    let dossier = std::env::temp_dir().join("ozalid-vignettes");
    std::fs::create_dir_all(&dossier).map_err(|e| format!("vignette impossible : {e}"))?;
    let images = images_vignette(&o.projet.images, &m.images);
    let (une, _) = package::ecrire_table(&images, &dossier)?;

    let src = couverture::source_une(
        &o.projet.meta.livre,
        &m.couverture,
        pr.format,
        une.as_ref(),
        dos_mm,
    );
    // Un fichier par maquette : la clé est celle d'une archive qui existe — `par_cle`
    // vient de la refuser sinon —, donc un simple nom de fichier.
    let typ = dossier.join(format!("{cle}.typ"));
    let png = dossier.join(format!("{cle}.png"));
    ecrire(&typ, &src)?;
    typst()?.apercu(&typ, &png, 1, 60)?;
    donnee_png(&png)
}

/// Enregistre la couverture du projet ouvert comme maquette personnalisée.
///
/// Le projet n'est pas touché : ce geste écrit à côté, dans le répertoire de
/// configuration, et ne rend donc aucune `ProjetVue`. La fenêtre rafraîchit sa liste en
/// rappelant `maquettes_liste`, seule source de vérité.
#[tauri::command]
pub fn maquette_enregistrer(
    nom: String,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<(), String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let cv = o
        .projet
        .meta
        .couverture
        .maquette
        .as_ref()
        .ok_or("aucune maquette : en choisir une avant de l'enregistrer.")?;
    maquettes::ecrire(&dir, &nom, cv, &o.projet.images)
}

/// Clone une maquette, fournie ou non, sous un nom que le Rust fabrique.
///
/// Aucun nom n'est demandé : « Bandeau (copie) » convient neuf fois sur dix, et
/// « Renommer » est à côté pour la dixième. Faire saisir ce nom aurait obligé le
/// dialogue à se donner un mode — un champ qui veut dire tantôt « enregistrer », tantôt
/// « cloner ceci ».
#[tauri::command]
pub fn maquette_cloner(cle: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    let m =
        maquettes::par_cle(Some(&dir), &cle).ok_or_else(|| format!("maquette inconnue : {cle}"))?;
    let nom = maquettes::nom_de_copie(Some(&dir), &m.nom);
    maquettes::ecrire(&dir, &nom, &m.couverture, &m.images)
}

/// Renomme une personnalisée. Le refus sur une fournie est dans `maquettes`, pas ici :
/// c'est lui la garantie, l'interface ne fait que ne pas offrir le bouton.
#[tauri::command]
pub fn maquette_renommer(cle: String, nom: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    maquettes::renommer(&dir, &cle, &nom)
}

#[tauri::command]
pub fn maquette_effacer(cle: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    maquettes::effacer(&dir, &cle)
}

#[tauri::command]
pub fn couverture_modifier(
    couverture: Couverture,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.meta.couverture.maquette = Some(couverture);
    vue_modifiee(o)
}

/// Nom sous lequel une image entre dans le projet, selon la face qu'elle sert.
///
/// Le nom porte le rôle — c'est ainsi que la composition le lit — et l'extension
/// vient du fichier choisi, parce que Typst distingue le PNG du JPEG.
fn nom_image(face: &str, ext: &str) -> Result<String, String> {
    match face {
        "une" => Ok(format!("couverture.{ext}")),
        "quatre" => Ok(format!("quatrieme.{ext}")),
        autre => Err(format!("face inconnue : {autre}")),
    }
}

/// Remplace l'image d'une face par un fichier choisi.
///
/// Le projet est auto-portant : l'image y est copiée, comme le manuscrit. Elle est
/// refusée ici plutôt qu'à la composition — une image dont Typst ne saura rien faire
/// n'a pas à entrer dans un `.ozalid` qui l'emporterait partout ensuite.
#[tauri::command]
pub fn image_choisir(
    face: String,
    chemin: String,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let source = Path::new(&chemin);
    let ext = source
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .filter(|e| matches!(e.as_str(), "jpg" | "jpeg" | "png"))
        .ok_or("image refusée : seuls le JPEG et le PNG se composent.")?;
    let nom = nom_image(&face, &ext)?;
    let octets = std::fs::read(source).map_err(|e| format!("image illisible : {e}"))?;
    Ressource::depuis(&nom, &octets)
        .ok_or_else(|| format!("{nom} : dimensions illisibles (ni PNG ni JPEG)."))?;

    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    poser_image(&mut o.projet.images, nom, octets);
    vue_modifiee(o)
}

/// Pose l'image d'une face et retire celle qui tenait déjà ce rôle.
///
/// Le remplacement se fait par rôle, pas par nom : une image importée s'appelle comme
/// elle veut, et deux images qui servent la même face laisseraient l'ordre alphabétique
/// décider laquelle se compose.
fn poser_image(images: &mut BTreeMap<String, Vec<u8>>, nom: String, octets: Vec<u8>) {
    let quatre = package::sert_la_quatrieme(&nom);
    images.retain(|n, _| package::sert_la_quatrieme(n) != quatre);
    images.insert(nom, octets);
}

/// Complète les images du livre par celles de la maquette, sans jamais en remplacer.
///
/// Une maquette porte une mise en page ; les photos qu'elle emporte ne sont qu'un
/// **repli**, pour le livre qui n'en a pas encore. Un livre déjà illustré garde donc les
/// siennes : c'est l'inverse de [`poser_image`], et c'est voulu — la photo appartient au
/// livre, la maquette n'est que la façon dont il paraît.
///
/// Le comblement se fait rôle par rôle, comme le remplacement : une maquette qui porte
/// les deux faces peut n'en poser qu'une, sur un livre qui n'avait que sa 1ère.
///
/// La règle ne regarde pas d'où vient l'archive, et c'est ce qui la rend cohérente :
/// une fournie et une personnalisée qui portent chacune une 1ère se comportent pareil
/// devant un livre illustré — elles s'effacent.
fn combler_images(projet: &mut BTreeMap<String, Vec<u8>>, maquette: &BTreeMap<String, Vec<u8>>) {
    for (nom, octets) in maquette {
        let quatre = package::sert_la_quatrieme(nom);
        if projet
            .keys()
            .any(|n| package::sert_la_quatrieme(n) == quatre)
        {
            continue;
        }
        projet.insert(nom.clone(), octets.clone());
    }
}

/// Les images sous lesquelles une maquette se *verrait*, sans que le projet bouge.
///
/// Une vignette doit montrer ce que choisir la maquette donnerait, pas ce que l'archive
/// contient : la règle est donc exactement celle de [`maquette_choisir`] — le comblement
/// de [`combler_images`] —, et c'est de la partager qui garantit que la vignette ne ment
/// pas. Une maquette purement typographique n'emporte aucune photo, et composée seule
/// elle montrerait une couverture nue là où la choisir aurait gardé celle du livre.
fn images_vignette(
    projet: &BTreeMap<String, Vec<u8>>,
    maquette: &BTreeMap<String, Vec<u8>>,
) -> BTreeMap<String, Vec<u8>> {
    let mut fondues = projet.clone();
    combler_images(&mut fondues, maquette);
    fondues
}

/// Retire du projet la photo que ce nom désigne.
///
/// Par nom et non par rôle, contrairement à [`poser_image`] : la fenêtre montre les noms
/// que cette même vue vient de lui servir, et c'est l'un d'eux qu'on clique. Un nom
/// absent est refusé plutôt qu'ignoré — il dit une liste périmée, donc un geste qui a
/// porté sur autre chose que ce qu'on voyait.
fn retirer_image(images: &mut BTreeMap<String, Vec<u8>>, nom: &str) -> Result<(), String> {
    images
        .remove(nom)
        .map(|_| ())
        .ok_or_else(|| format!("{nom} : le projet ne porte pas cette image."))
}

/// Retire une photo du projet, et la retire donc du `.ozalid`.
///
/// C'est le seul geste qui allège l'archive : régler le fond de la 4ème sur le papier de
/// la 1ère cesse de composer la photo, mais elle reste embarquée — et une photo
/// d'appareil pèse plus que le manuscrit.
///
/// La maquette n'est pas touchée : un fond réglé sur « Image propre » le reste, et
/// compose alors son papier seul. Le corriger d'autorité déciderait à la place de qui
/// remplace une photo par une autre en deux gestes.
#[tauri::command]
pub fn image_retirer(nom: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    retirer_image(&mut o.projet.images, &nom)?;
    vue_modifiee(o)
}

/// Ce qu'un aperçu de face donne à voir : l'image, et où la planche se coupe et se plie
/// s'il y a lieu.
#[derive(Serialize)]
pub struct Apercu {
    pub image: String,
    /// Absents sur les faces qui se composent au format rogné, sans fond perdu — la
    /// 1ère, la 4ème et le dos. C'est le Rust qui l'affirme plutôt que la fenêtre qui
    /// le déduise d'un nom de face : le jour où une face gagne du fond perdu, elle
    /// gagne ses repères sans qu'on y pense.
    pub reperes: Option<Reperes>,
    /// Ce que la planche mesure, en millimètres, pour l'écrire sous l'aperçu.
    ///
    /// Séparé des repères, avec lesquels il voyage pourtant toujours : les repères sont
    /// des fractions posées **sur** l'image, ceux-ci des millimètres écrits **sous**
    /// elle. Les confondre ferait porter à l'habillage une unité qui n'y survit pas.
    pub mesures: Option<Mesures>,
}

/// Les quatre mesures d'une planche, en millimètres.
///
/// Elles ne se recalculent pas dans la fenêtre : la largeur d'une planche est deux
/// couvertures, un dos et deux fonds perdus, et cette règle est déjà écrite une fois,
/// dans `planche::Gabarit`. Redite en JavaScript, elle dériverait le jour où un
/// imprimeur compterait autrement — et le chiffre affiché ne serait plus celui du
/// fichier remis.
#[derive(Serialize)]
pub struct Mesures {
    pub largeur: f64,
    pub hauteur: f64,
    pub dos: f64,
    pub fond_perdu: f64,
}

/// Où la planche se coupe et où elle se plie, en fraction de ses propres dimensions.
///
/// `x` et `y` sont la part du fond perdu sur la largeur puis sur la hauteur ; `pli_quatre`
/// et `pli_une` sont les deux plis qui encadrent le dos, comptés depuis le bord gauche.
/// Les quatre voyagent ensemble parce qu'ils s'affichent ensemble, et qu'aucun n'existe
/// sans les autres : ce sont les repères d'une planche, ceux-là mêmes que le PDF remis
/// à l'imprimeur ne porte pas.
#[derive(Serialize)]
pub struct Reperes {
    pub x: f64,
    pub y: f64,
    pub pli_quatre: f64,
    pub pli_une: f64,
}

/// Aperçu d'une face de couverture ou de la planche entière, en PNG encodé dans une
/// URL `data:`.
///
/// L'aperçu sort du **même** moteur et de la même source que le PDF final : il n'y a
/// donc pas d'écart écran/export à surveiller, contrairement à l'atelier HTML.
///
/// `dos_mm` vient de la dernière composition de l'intérieur ; il n'est jamais saisi.
/// Sans lui, la planche ne s'aperçoit pas — c'est voulu : une planche dont le dos
/// serait deviné donnerait à voir un livre qui n'existe pas.
#[tauri::command]
pub fn couverture_apercu(
    face: String,
    dos_mm: Option<f64>,
    atelier: State<Atelier>,
) -> Result<Apercu, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let cv = o
        .projet
        .meta
        .couverture
        .maquette
        .as_ref()
        .ok_or("aucune maquette : en choisir une.")?;
    // Le format vient du livrable visé, et le fond perdu de son relevé quand
    // l'imprimeur n'en publie pas : les deux sont dans le projet, plus dans un champ.
    let (pr, _, d) = vise(o)?;
    let fond_perdu_mm = d.fond_perdu_mm;

    // Répertoire de travail de l'aperçu : temporaire, jamais à côté du projet. Un
    // aperçu n'est pas une sortie, et il est réécrit à chaque réglage.
    let dossier = std::env::temp_dir().join("ozalid-apercu");
    std::fs::create_dir_all(&dossier).map_err(|e| format!("aperçu impossible : {e}"))?;

    let (une, quatre) = ecrire_images(&o.projet, &dossier)?;
    // Seule la planche se compose avec du fond perdu : les trois autres faces n'ont
    // rien à faire marquer.
    let mut reperes = None;
    let mut mesures = None;
    let src = match face.as_str() {
        "une" => couverture::source_une(&o.projet.meta.livre, cv, pr.format, une.as_ref(), dos_mm),
        "quatre" => couverture::source_quatre(
            &o.projet.meta.livre,
            cv,
            pr.format,
            quatre.as_ref(),
            une.as_ref(),
            dos_mm,
        )?,
        // Le dos seul se compose sans fond perdu : il ne réclame donc que la
        // pagination, là où la planche réclame aussi le gabarit de l'imprimeur.
        "dos" => {
            let dos = dos_mm.ok_or(
                "dos : composer l'intérieur d'abord, c'est la pagination qui donne le dos.",
            )?;
            planche::source_dos(&o.projet.meta.livre, cv, pr.format, dos, une.as_ref())
        }
        "planche" => {
            let dos = dos_mm.ok_or(
                "planche : composer l'intérieur d'abord, c'est la pagination qui donne le dos.",
            )?;
            let fp = pr.fond_perdu.or(fond_perdu_mm).ok_or_else(|| {
                format!(
                    "{} ne publie pas de fond perdu : le relever sur son gabarit et le saisir.",
                    pr.libelle
                )
            })?;
            let g = planche::Gabarit {
                format: pr.format,
                dos,
                fond_perdu: fp,
            };
            let (x, y) = g.part_fond_perdu();
            let (pli_quatre, pli_une) = g.plis();
            reperes = Some(Reperes {
                x,
                y,
                pli_quatre,
                pli_une,
            });
            mesures = Some(Mesures {
                largeur: g.largeur(),
                hauteur: g.hauteur(),
                dos: g.dos,
                fond_perdu: g.fond_perdu,
            });
            planche::source(&o.projet.meta.livre, cv, &g, une.as_ref(), quatre.as_ref())?
        }
        autre => return Err(format!("face inconnue : {autre}")),
    };

    let typ = dossier.join(format!("apercu-{face}.typ"));
    let png = dossier.join(format!("apercu-{face}.png"));
    ecrire(&typ, &src)?;
    typst()?.apercu(&typ, &png, 1, 150)?;

    Ok(Apercu {
        image: donnee_png(&png)?,
        reperes,
        mesures,
    })
}

/// De quoi montrer la photo bouger sous la souris sans rien recomposer.
///
/// Trois pièces qui, empilées dans cet ordre, refont la face à l'identique : le papier,
/// la photo dans sa zone, et l'habillage par-dessus. La fenêtre n'a plus qu'à déplacer
/// la pièce du milieu — et ce qu'elle montre pendant le geste est ce que Typst
/// composera, pas une approximation.
///
/// Le prix est d'une composition de plus, demandée **après** l'aperçu et jamais pendant
/// un geste : l'habillage ne dépend pas du cadrage, il vaut donc pour le geste entier.
#[derive(Serialize)]
pub struct Calques {
    /// La face composée sans son papier ni sa photo, en PNG à fond transparent : le
    /// voile, le cadre, les textes, la pastille — tout ce qui se pose *par-dessus*
    /// l'image, et rien d'autre.
    ///
    /// Ce n'est pas une source de plus : c'est la même, composée sur un papier
    /// transparent et sans photo. Une deuxième façon d'écrire une couverture finirait
    /// par montrer autre chose que ce qui s'imprime.
    pub habillage: String,
    /// La photo telle que le projet la porte, en donnée `data:`.
    pub photo: String,
    pub naturel_l: u32,
    pub naturel_h: u32,
    /// La zone où la photo se compose, en fraction de la face — la seule unité qui
    /// survive à un aperçu affiché à la taille que la fenêtre lui laisse.
    pub zone: Zone,
    /// Le papier de cette face-là : la 4ème peut avoir le sien.
    pub papier: String,
}

#[derive(Serialize)]
pub struct Zone {
    pub x: f64,
    pub y: f64,
    pub l: f64,
    pub h: f64,
}

/// Le papier rendu transparent : c'est ce qui distingue l'habillage de la face entière.
const PAPIER_TRANSPARENT: &str = "#00000000";

/// Les calques d'une face manipulable, ou `None` s'il n'y a pas de photo à y déplacer.
///
/// Deux faces seulement : la 1ère et la 4ème. Le dos ne se cadre pas — sa photo, quand
/// il en porte une, est la tranche du prolongement de la 1ère, et se règle là-bas —, et
/// la planche est une vue de contrôle qui ne règle rien.
#[tauri::command]
pub fn couverture_calques(
    face: String,
    dos_mm: Option<f64>,
    atelier: State<Atelier>,
) -> Result<Option<Calques>, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let Some(cv) = o.projet.meta.couverture.maquette.as_ref() else {
        return Ok(None);
    };
    let (pr, _, _) = vise(o)?;
    let format = pr.format;
    let b = couverture::Boite::rognee(format);

    let dossier = std::env::temp_dir().join("ozalid-apercu");
    std::fs::create_dir_all(&dossier).map_err(|e| format!("aperçu impossible : {e}"))?;
    let (une, quatre) = ecrire_images(&o.projet, &dossier)?;

    // La zone et la photo, demandées au moteur de composition plutôt que déduites d'un
    // nom de mode : c'est ce qui garantit que la souris cadre là où Typst composera.
    let pano = couverture::panorama_face(format, dos_mm, true);
    let (zone, r, papier) = match face.as_str() {
        "une" => {
            let Some(r) = une.as_ref() else {
                return Ok(None);
            };
            match couverture::image_une(cv, format, Some(r), b, pano) {
                Some((zone, _)) => (zone, r, cv.papier.clone()),
                None => return Ok(None),
            }
        }
        // La 4ème ne se cadre à la souris que lorsqu'elle porte sa propre image. En
        // prolongement, son cadrage est celui de la 1ère — le panneau le dit déjà, et
        // offrir ici une poignée qui déplacerait la photo de l'autre face serait un
        // piège. Papier hérité et couleur distincte n'ont, eux, rien à déplacer.
        "quatre" if cv.quatrieme.fond == couverture::FondQuatre::Image => {
            match couverture::photo_quatre(cv, format, quatre.as_ref(), None, None, b)? {
                Some((zone, _, r)) => (zone, r, couverture::papier_quatre(cv).to_string()),
                None => return Ok(None),
            }
        }
        _ => return Ok(None),
    };

    // Les octets de la photo, retrouvés par le nom que la composition vient d'employer :
    // le projet ne range pas ses images sous un rôle mais sous leur nom de fichier.
    let octets = o
        .projet
        .images
        .get(&r.fichier)
        .ok_or_else(|| format!("{} : image absente du projet.", r.fichier))?;

    let mut nu = cv.clone();
    nu.papier = PAPIER_TRANSPARENT.to_string();
    nu.quatrieme.couleur = PAPIER_TRANSPARENT.to_string();
    // La 4ème s'assemble à la main, préambule et corps séparés, pour y glisser son
    // voile : celui-ci suit désormais la photo réellement composée, et l'habillage se
    // compose sans photo — c'est tout son objet, la laisser bouger dessous. Sans cette
    // reprise, le direct montrerait une photo nue et l'aperçu une photo voilée.
    //
    // Entre les deux et non avant : un `#set page` qui suit du contenu ouvre une page de
    // plus. Et entre les deux met bien le voile sous les textes et sous le rectangle de
    // fond — lequel est transparent ici, et une couleur d'alpha nul ne masque rien.
    //
    // La 1ère n'a rien à reprendre : son voile suit le mode de la page, qui vaut pour
    // l'habillage comme pour la face entière.
    let corps = match face.as_str() {
        "une" => couverture::source_une(&o.projet.meta.livre, &nu, format, None, dos_mm),
        _ => {
            let pano = couverture::panorama_face(format, dos_mm, false);
            couverture::preambule(b.largeur, b.hauteur)
                + &couverture::bloc_voile(b, nu.quatrieme.voile, nu.quatrieme.voile_opacite)
                + &couverture::corps_quatre(&o.projet.meta.livre, &nu, format, None, None, pano, b)?
        }
    };
    // `#set page(fill: none)` en tête : une règle posée avant le préambule vaut avec
    // lui, et c'est elle qui rend le PNG transparent là où le papier ne peint plus.
    let src = format!("#set page(fill: none)\n{corps}");
    let typ = dossier.join(format!("habillage-{face}.typ"));
    let png = dossier.join(format!("habillage-{face}.png"));
    ecrire(&typ, &src)?;
    typst()?.apercu(&typ, &png, 1, 150)?;

    Ok(Some(Calques {
        habillage: donnee_png(&png)?,
        photo: donnee_image(octets),
        naturel_l: r.largeur,
        naturel_h: r.hauteur,
        zone: Zone {
            x: zone.0 / b.largeur,
            y: zone.1 / b.hauteur,
            l: zone.2 / b.largeur,
            h: zone.3 / b.hauteur,
        },
        papier,
    }))
}

/// Où chaque élément du dos tombe sur l'aperçu couché, pour pouvoir l'y saisir.
///
/// Vide quand le dos ne porte aucun texte — un dos nu n'a rien à réorganiser. Séparé de
/// l'aperçu parce qu'il coûte une évaluation de plus et qu'il ne sert qu'une face : le
/// demander à chaque composition ferait payer la mesure du dos à la planche, qui n'en
/// fait rien.
#[tauri::command]
pub fn couverture_dos_boites(
    dos_mm: Option<f64>,
    atelier: State<Atelier>,
) -> Result<Vec<planche::BoiteDos>, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let Some(cv) = o.projet.meta.couverture.maquette.as_ref() else {
        return Ok(Vec::new());
    };
    // Le dos ne change pas les longueurs mesurées — elles suivent la largeur de
    // couverture — mais l'aperçu qu'on habille n'existe pas sans lui, et la fenêtre ne
    // doit pas poser de prises sur une face qu'elle n'a pas pu composer.
    if dos_mm.is_none() {
        return Ok(Vec::new());
    }
    let (pr, _, _) = vise(o)?;
    let livre = &o.projet.meta.livre;
    let src = planche::source_mesures(livre, cv, pr.format);
    // Aucun texte à mesurer : Typst rendrait un objet vide, autant ne pas le déranger.
    if !src.contains("measure(") {
        return Ok(Vec::new());
    }
    let dossier = std::env::temp_dir().join("ozalid-apercu");
    std::fs::create_dir_all(&dossier).map_err(|e| format!("aperçu impossible : {e}"))?;
    let typ = dossier.join("mesures-dos.typ");
    ecrire(&typ, &src)?;
    let mesures = typst()?.mesures(&typ)?;
    Ok(planche::boites_dos(livre, cv, pr.format, &mesures))
}

/// Un PNG du disque, en donnée `data:` : la fenêtre ne lit pas les fichiers, une image
/// n'y entre pas autrement.
fn donnee_png(chemin: &Path) -> Result<String, String> {
    let octets = std::fs::read(chemin).map_err(|e| format!("aperçu illisible : {e}"))?;
    Ok(donnee_image(&octets))
}

/// Des octets d'image, prêts à poser dans une balise `img`.
///
/// Le type est relevé sur le contenu : la fenêtre affiche d'après lui, et un JPEG
/// annoncé en PNG resterait un cadre vide.
fn donnee_image(octets: &[u8]) -> String {
    let type_mime = match crate::image::extension(octets) {
        Some("jpg") => "image/jpeg",
        _ => "image/png",
    };
    format!(
        "data:{type_mime};base64,{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, octets)
    )
}

/* ---------- packages ---------- */

/// Ce que rend la génération pour un livrable : le package, ou l'erreur qui l'a
/// empêché. Un livrable en échec n'interrompt pas les autres — mais il est dit.
#[derive(Serialize)]
pub struct Resultat {
    /// L'identité du livrable, à quatre axes : c'est elle qui nomme son répertoire.
    pub cle: String,
    pub libelle: String,
    /// La finition retenue, sous son nom d'imprimeur. Absente le plus souvent : c'est
    /// le cas courant, et l'écran ne montre pas une ligne vide.
    pub finition: Option<String>,
    pub package: Option<package::Package>,
    /// La planche du package, en PNG, prête à poser dans une balise `img`.
    ///
    /// Le chemin du fichier ne suffirait pas : la fenêtre ne lit pas le disque, et
    /// c'est déjà par une donnée en clair que l'aperçu de la Couverture voyage.
    pub vignette: Option<String>,
    pub erreur: Option<String>,
}

/// Le nom d'imprimeur de la finition retenue, tel que le récapitulatif la porte.
///
/// La clé du `.ozalid` ne se montre pas : c'est « Pelliculage mat » qu'on coche sur un
/// bon de commande, pas « mat ». À défaut de nom — un POD inconnu, une finition que le
/// catalogue ne porte plus et que `normalise` n'élague pas —, la clé telle quelle : un
/// mot brut se lit encore, une ligne absente ne se lit pas.
fn nom_finition(livrable: &Livrable, pod: Option<&catalogue::Pod>) -> Option<String> {
    let cle = livrable.finition.as_deref()?;
    // La règle vit sur le `Pod` : la fiche de téléversement la lit au même endroit, et
    // deux résolutions d'une même clé finiraient par ne plus donner le même nom.
    Some(pod.map_or_else(|| cle.to_owned(), |p| p.nom_finition(cle)))
}

/// Ce que rend la génération : les packages, et le projet tel qu'elle l'a laissé.
///
/// La vue voyage avec, comme celle de `composer` : générer **compose**, et une commande
/// qui écrit dans le projet en rend la vue — c'est la règle de toutes les autres.
#[derive(Serialize)]
pub struct Generation {
    pub projet: ProjetVue,
    pub packages: Vec<Resultat>,
}

/// Génère le package de chaque livrable du livre, chacun dans son répertoire.
///
/// Une seule maquette, N livrables, aucun réglage retouché entre eux : chacun
/// compose son propre intérieur, donc sa propre pagination, donc son propre dos. C'est
/// la promesse de l'étape Livraison, et la liste vient du projet — plus de cases à
/// cocher qui désigneraient les livrables une seconde fois.
#[tauri::command]
pub fn packager(atelier: State<Atelier>) -> Result<Generation, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let livrables = o.projet.meta.livraison.livrables.clone();
    if livrables.is_empty() {
        return Err("aucun livrable : en déclarer un.".into());
    }
    let typst = typst()?;

    // Résolution d'abord : un axe ou un papier inconnu se fige en `Resultat` d'erreur
    // ici, sans passer par le lot. Le reste devient une `Cible`, dans l'ordre des
    // livrables — c'est cet ordre que la fin de la fonction restitue.
    let mut etapes: Vec<Result<package::Cible, Resultat>> = Vec::with_capacity(livrables.len());
    for d in &livrables {
        etapes.push(match catalogue::resout(&d.fabrication) {
            Ok(r) => Ok(cible(r.provider(), r.papier.clone(), d)),
            // Le POD est le seul axe qui puisse encore se nommer quand la résolution
            // échoue sur un autre : afficher la clé à quatre segments en gros titre
            // serait un recul devant « BoD ».
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

    // `?` fait échouer la commande entière, sans `Resultat` par livrable : à la
    // différence d'un POD ou d'un papier inconnu, une racine de sorties
    // inutilisable (projet non enregistré) ne concerne aucun livrable en
    // particulier, et rien ne peut être tenté avant qu'elle existe.
    let racine = sorties_racine(o)?;
    let cibles: Vec<package::Cible> = etapes
        .iter()
        .filter_map(|e| e.as_ref().ok().cloned())
        .collect();
    let mut paquets = package::lot(&o.projet, &cibles, &racine, &typst).into_iter();

    // `zip` sur les livrables : `etapes` a été poussée dans leur ordre, et la finition
    // ne voyage pas dans la `Cible` — elle ne fabrique rien, aucun octet du PDF ni aucun
    // nom de fichier n'en dépend. Elle se commande, et le récapitulatif est le seul
    // endroit où elle peut être lue.
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
                        // La vignette manquante ne perd pas le package : les PDF sont
                        // écrits, et c'est eux que l'imprimeur reçoit.
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

    // Ce que la génération vient de mesurer entre dans le projet, gabarit par gabarit,
    // exactement comme la mesure de `composer` : c'est le même livre, composé par le
    // même Typst, sous la même clé de rangement. Sans cela le pied restait sur « dos non
    // composé » pendant que le compte rendu, deux centimètres plus haut, donnait le dos
    // — deux mesures du même livre, une seule affichée, et c'était celle qui manquait.
    //
    // Le consentement ne s'y oppose pas : il gouverne le déclenchement d'une composition
    // que personne n'a demandée, pas le droit de retenir celle qu'un clic vient de
    // réclamer. `retenir_mesure` ignore de lui-même un gabarit que plus aucun livrable
    // ne porte.
    for (d, r) in livrables.iter().zip(&sorties) {
        let (Some(p), Ok(resolu)) = (&r.package, catalogue::resout(&d.fabrication)) else {
            continue;
        };
        o.projet.meta.livraison.retenir_mesure(
            &d.fabrication.cle_gabarit(),
            Mesure {
                pages: p.pages,
                gouttiere: p.gouttiere,
                blanche: p.blanche,
                empreinte: Some(resolu.empreinte()),
                polices_introuvables: p.polices_introuvables.clone(),
            },
        );
    }

    Ok(Generation {
        projet: vue_modifiee(o)?,
        packages: sorties,
    })
}

/// Génère les ebooks locaux dans `<projet>/ebook/`.
///
/// Une livraison, mais locale : elle ne vise aucun imprimeur, elle emprunte seulement
/// le gabarit de celui qui est visé — c'est de là que viennent le format, le corps et
/// l'interligne, faute d'un format d'écran qui voudrait dire quelque chose.
#[tauri::command]
pub fn ebook_generer(atelier: State<Atelier>) -> Result<ebook::Ebooks, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (pr, _, d) = vise(o)?;
    let dossier = sorties_racine(o)?.join("ebook");
    ebook::generer(&o.projet, &pr, d.dos_mm, &dossier, &typst()?)
}

/* ---------- envois ---------- */

/// Ce qu'un envoi produit, du point de vue de l'interface.
#[derive(Serialize)]
pub struct ResultatEnvoi {
    pub dedicataire: String,
    /// Nom du répertoire écrit sous `envois/` — assaini, donc pas toujours celui du
    /// dédicataire. C'est celui-là qu'il faut ouvrir, et donc celui-là qu'on montre.
    pub dossier: String,
    pub package: package::Package,
    pub vignette: Option<String>,
}

/// Remplace un envoi par lui-même modifié : sa main, son mot, son placement.
///
/// Un envoi et non la liste entière, contrairement à `envois_modifier` qui l'a
/// précédée : celle-ci recevait l'objet entier, si bien que ce que le front n'envoyait
/// pas était effacé — une main omise revenait au défaut, et vingt exemplaires
/// changeaient d'écriture sans que personne ne l'ait demandé. Ici le rang désigne, et
/// le reste ne bouge pas.
#[tauri::command]
pub fn envoi_regler(
    index: usize,
    envoi: crate::envoi::Envoi,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let mut envois = o.projet.meta.envois.clone();
    *envois
        .liste
        .get_mut(index)
        .ok_or("envoi introuvable : la liste a changé.")? = envoi;
    o.projet.regler_envois(envois)?;
    vue_modifiee(o)
}

/// Ajoute un envoi, qui naît comme le précédent.
///
/// La règle vit dans `Envois::ajouter`, avec le modèle : c'est une propriété du livre,
/// et non de la façon dont l'interface la demande.
#[tauri::command]
pub fn envoi_ajouter(dedicataire: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let mut envois = o.projet.meta.envois.clone();
    envois.ajouter(dedicataire);
    o.projet.regler_envois(envois)?;
    vue_modifiee(o)
}

/// Retire un envoi.
///
/// Son image s'en va avec lui : c'est `regler_envois` qui élague ce que plus aucun
/// envoi ne nomme, sans quoi l'archive garderait le mot manuscrit d'une personne à qui
/// l'on n'envoie plus rien.
#[tauri::command]
pub fn envoi_retirer(index: usize, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let mut envois = o.projet.meta.envois.clone();
    if index >= envois.liste.len() {
        return Err("envoi introuvable : la liste a changé.".into());
    }
    envois.liste.remove(index);
    o.projet.regler_envois(envois)?;
    vue_modifiee(o)
}

/// Le gabarit de diffusion, partagé par tous les envois du livre.
///
/// Au livre et non à l'envoi : c'est le style d'écriture du tirage, dans lequel le mot
/// de chacun s'insère. Le réécrire pour chaque personne n'aurait pas d'usage.
#[tauri::command]
pub fn envois_gabarit(gabarit: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    regler_style(&atelier, |e| e.gabarit = gabarit)
}

/// La couleur de l'encre que `{couleur}` nomme au modèle.
///
/// Au livre comme le gabarit : un auteur signe ses vingt exemplaires du même stylo.
#[tauri::command]
pub fn envois_couleur(couleur: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    regler_style(&atelier, |e| e.couleur = couleur)
}

/// Le paraphe de l'auteur, que `{paraphe}` nomme au modèle.
///
/// À ne pas confondre avec le `monogramme` du livre, qui nomme la **maison** et figure
/// au pied de la couverture. Celui-ci est une signature manuscrite.
#[tauri::command]
pub fn envois_paraphe(paraphe: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    regler_style(&atelier, |e| e.paraphe = paraphe)
}

/// Applique une retouche au style d'écriture du livre, et rend la vue.
///
/// Les trois réglages ne diffèrent que par le champ qu'ils touchent : les écrire trois
/// fois en entier ferait diverger leurs contrôles au premier ajout.
fn regler_style(
    atelier: &State<Atelier>,
    retouche: impl FnOnce(&mut crate::envoi::Envois),
) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let mut envois = o.projet.meta.envois.clone();
    retouche(&mut envois);
    o.projet.regler_envois(envois)?;
    vue_modifiee(o)
}

/// Le gabarit de départ des projets neufs, tel que les préférences le portent.
#[tauri::command]
pub fn gabarit_defaut_lire(app: tauri::AppHandle) -> String {
    gabarit_de_depart(&app)
}

/// Retient ce gabarit comme départ des projets neufs.
///
/// **Ne touche pas au livre ouvert** : c'est un réglage de la machine, et le gabarit qui
/// compose reste celui du `.ozalid`. Les deux se règlent au même endroit à l'écran, ils
/// ne vivent pas au même endroit sur le disque.
#[tauri::command]
pub fn gabarit_defaut_poser(gabarit: String, app: tauri::AppHandle) -> Result<(), String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    let mut p = preferences::charger(&dir);
    p.gabarit_defaut = gabarit;
    preferences::enregistrer(&dir, &p)
}

/// Les mains offertes par l'application.
///
/// La police personnelle n'y est pas : elle appartient au livre ouvert, pas à
/// l'application, et le front la lit dans `envois.personnelle`.
#[tauri::command]
pub fn mains_liste() -> Vec<&'static str> {
    crate::envoi::MAINS.to_vec()
}

/// Embarque la police manuscrite de l'auteur dans le projet, et en fait sa main.
///
/// Le fichier est copié dans le `.ozalid`, comme le manuscrit et les photos : le projet
/// doit composer à l'identique sur une machine où cette écriture n'est installée nulle
/// part. C'est aussi pourquoi la famille est relevée dans le fichier plutôt que déduite
/// de son nom.
#[tauri::command]
pub fn police_choisir(chemin: String, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let source = Path::new(&chemin);
    // Typst ne charge d'un répertoire de polices que les fichiers dont l'extension le
    // dit. Une écriture rangée sous un autre nom n'y serait jamais lue, et l'envoi
    // partirait dans la police de repli sans qu'aucun message ne le signale.
    let nom = source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| {
            let bas = n.to_lowercase();
            bas.ends_with(".ttf") || bas.ends_with(".otf")
        })
        .ok_or("police refusée : seuls les fichiers .ttf et .otf se composent.")?;
    let octets = std::fs::read(source).map_err(|e| format!("police illisible : {e}"))?;

    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.poser_police(&nom, octets)?;
    vue_modifiee(o)
}

/// Embarque l'image écrite à la main pour un envoi.
///
/// Elle entre dans le `.ozalid` sous `envois/`, et non avec les photos de couverture :
/// là-bas, une image dont le nom ne commence pas par `quatrieme` devient la première de
/// couverture — le mot manuscrit d'un lecteur remplacerait la couverture du livre.
#[tauri::command]
pub fn envoi_image_choisir(
    index: usize,
    chemin: String,
    atelier: State<Atelier>,
) -> Result<ProjetVue, String> {
    // Aucun contrôle sur l'extension du fichier choisi : c'est le contenu qui décide,
    // et `poser_image_envoi` le relève. Une photo d'appareil renommée en `.png` reste
    // un JPEG, et Typst la lirait à son nom.
    let octets = std::fs::read(Path::new(&chemin)).map_err(|e| format!("image illisible : {e}"))?;

    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.poser_image_envoi(index, octets)?;
    vue_modifiee(o)
}

/// Ce que l'interface sait de l'accès au modèle de diffusion.
///
/// **La clé n'y est pas.** Elle est en clair dans `preferences.toml`, avec les
/// permissions du fichier ; la renvoyer au front la ferait entrer dans une page, donc
/// dans une capture d'écran, donc dans un message. Savoir qu'elle est posée suffit à
/// régler l'accès.
#[derive(Serialize)]
pub struct AccesVue {
    pub url: String,
    pub cle_posee: bool,
    /// Le nom du modèle, quand le fournisseur l'attend dans le corps. Contrairement à
    /// la clé, il revient à l'interface : ce n'est pas un secret, et un champ qui se
    /// rouvre vide se ressaisit de travers.
    pub modele: String,
}

#[tauri::command]
pub fn diffusion_lire(app: tauri::AppHandle) -> AccesVue {
    let d = config(&app).map(|c| preferences::charger(&c).diffusion);
    AccesVue {
        url: d.as_ref().map(|d| d.url.clone()).unwrap_or_default(),
        modele: d.as_ref().map(|d| d.modele.clone()).unwrap_or_default(),
        cle_posee: d.is_some_and(|d| !d.cle.trim().is_empty()),
    }
}

/// Règle l'accès au modèle. `cle` absente laisse en place celle qui est enregistrée.
///
/// Sans cela, corriger l'adresse effacerait la clé — le champ de saisie est vide à
/// l'écran, puisqu'on ne la lui redonne jamais.
#[tauri::command]
pub fn diffusion_regler(
    url: String,
    modele: String,
    cle: Option<String>,
    app: tauri::AppHandle,
) -> Result<AccesVue, String> {
    let dir = config(&app).ok_or("répertoire de configuration introuvable.")?;
    let mut p = preferences::charger(&dir);
    p.diffusion.url = url;
    p.diffusion.modele = modele;
    if let Some(c) = cle {
        p.diffusion.cle = c;
    }
    preferences::enregistrer(&dir, &p)?;
    Ok(diffusion_lire(app))
}

/// Demande au modèle l'image d'un envoi, et la garde de côté sans la figer.
///
/// Rendue en PNG encodé pour l'aperçu, et **pas** écrite dans le projet : un modèle de
/// diffusion rend rarement une écriture lisible du premier coup. On regarde, on
/// regénère, et c'est `envoi_accepter` qui fait entrer l'image dans l'archive.
#[tauri::command]
pub fn envoi_generer(
    index: usize,
    app: tauri::AppHandle,
    atelier: State<Atelier>,
) -> Result<String, String> {
    let acces = config(&app)
        .map(|c| preferences::charger(&c).diffusion)
        .unwrap_or_default();
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let e = o
        .projet
        .meta
        .envois
        .liste
        .get(index)
        .ok_or("envoi introuvable : la liste a changé.")?;
    // La main appartient à l'exemplaire depuis la v4 : c'est celle de **cet** envoi qui
    // décide, et non plus celle du livre. Le gabarit, lui, est resté au livre — c'est le
    // style d'écriture du tirage, pas le mot d'une personne.
    if !matches!(e.main, crate::envoi::Main::Diffusion) {
        return Err("la main de cet envoi n'est pas une image générée.".into());
    }
    let gabarit = &o.projet.meta.envois.gabarit;
    // Le titre vient du livre et non de l'envoi : il est le même pour tout le tirage.
    let mots = crate::diffusion::Mots {
        envoi: &e.contenu,
        dedicataire: &e.dedicataire,
        titre: &o.projet.meta.livre.titre,
        couleur: &o.projet.meta.envois.couleur,
        paraphe: &o.projet.meta.envois.paraphe,
    };

    let octets = crate::diffusion::genere(
        &acces,
        &crate::diffusion::prompt(gabarit, &mots),
        &crate::diffusion::Reseau,
    )?;
    let donnee = donnee_image(&octets);
    o.candidat = Some((index, octets));
    Ok(donnee)
}

/// Fige l'image générée : elle entre dans l'archive, et n'en bouge plus.
///
/// À partir d'ici, composer ne rappelle jamais le réseau — le package se refait des mois
/// plus tard, hors ligne, à l'identique.
#[tauri::command]
pub fn envoi_accepter(index: usize, atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    let (_, octets) = o
        .candidat
        .take()
        // Le candidat porte son index : accepter après avoir changé de ligne poserait
        // sinon l'image d'une personne sur l'exemplaire d'une autre.
        .filter(|(pour, _)| *pour == index)
        .ok_or("aucune image en attente pour cet envoi : en générer une.")?;
    o.projet.poser_image_envoi(index, octets)?;
    vue_modifiee(o)
}

/// Retire la police de l'auteur du projet.
#[tauri::command]
pub fn police_retirer(atelier: State<Atelier>) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_mut().ok_or_else(aucun_projet)?;
    o.projet.retirer_police();
    vue_modifiee(o)
}

/* ---------- ce que le canevas de placement regarde ---------- */

/// 120 px de large sur une page de 127 mm : de quoi reconnaître une page dans un rail.
const VIGNETTE_PPI: u32 = 24;
/// 750 px sur la même page : de quoi placer un envoi à la souris.
const PAGE_PPI: u32 = 150;
/// L'objet est agrandi par le canevas, et une signature pixelisée sous la souris ferait
/// douter du rendu.
const OBJET_PPI: u32 = 300;

/// La source de l'intérieur **sans envoi**, et le répertoire où elle est écrite.
///
/// Sans envoi parce qu'un `foreground` ne réordonne rien : la page de fond ne dépend
/// d'aucun dédicataire, et la même série de rendus sert à tous les exemplaires. C'est
/// aussi ce qui permet de glisser l'objet sans rappeler Typst — le fond ne bouge pas.
///
/// Le répertoire est nommé par l'empreinte de la source. Une composition qui change
/// change l'empreinte, donc le répertoire : il n'y a pas d'invalidation à écrire,
/// seulement un nom à calculer. Ce qui reste dort dans le temporaire du système, qui
/// est fait pour cela.
fn source_de_fond(o: &Ouvert) -> Result<(PathBuf, PathBuf), String> {
    let (pr, _, d) = vise(o)?;
    let int = &o.projet.meta.interieur;
    int.verifie()?;
    let livre = &o.projet.meta.livre;
    let chapitres = manuscrit::decoupe(&o.projet.texte, livre.chapitres)?;
    // La mesure du tirage, et non un réglage d'aperçu : les pages que le canevas montre
    // doivent être celles que le dédicataire recevra, numéros compris.
    let mesure = o
        .projet
        .meta
        .livraison
        .mesure(&d.fabrication.cle_gabarit())
        .ok_or("intérieur non composé : le placement a besoin des pages du tirage.")?;
    let reglage = Reglage {
        gouttiere: mesure.gouttiere,
        blanche: mesure.blanche,
    };
    let src = interieur::source(livre, int, &pr, &reglage, &chapitres, None);
    let dossier = std::env::temp_dir()
        .join("ozalid-pages")
        .join(empreinte(&src));
    std::fs::create_dir_all(&dossier)
        .map_err(|e| format!("répertoire inutilisable ({}) : {e}", dossier.display()))?;
    let chemin = dossier.join("fond.typ");
    ecrire(&chemin, &src)?;
    Ok((chemin, dossier))
}

/// Une empreinte courte et stable d'une source, pour nommer son répertoire de rendus.
///
/// `DefaultHasher` suffit : ce n'est pas un contrôle d'intégrité, seulement un nom qui
/// change quand la source change. Une collision coûterait des vignettes périmées, pas un
/// mauvais tirage.
fn empreinte(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Toutes les pages de l'intérieur en vignettes, pour le livrable visé.
///
/// Aucun cache : les 190 pages du livre témoin coûtent six dixièmes de seconde, et
/// l'interface ne demande cette série qu'à l'ouverture de l'étape. Un cache achèterait
/// ce dixième-là au prix d'une invalidation à tenir juste.
#[tauri::command]
pub fn envoi_vignettes(atelier: State<Atelier>) -> Result<Vec<String>, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (src, dossier) = source_de_fond(o)?;
    let pages = typst()?.apercus(&src, &dossier.join("v{p}.png"), VIGNETTE_PPI)?;
    pages.iter().map(|p| donnee_png(p)).collect()
}

/// Une page de l'intérieur, en grand, pour le canevas de placement.
#[tauri::command]
pub fn envoi_page(page: u32, atelier: State<Atelier>) -> Result<String, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (src, dossier) = source_de_fond(o)?;
    let png = dossier.join(format!("grand-{page}.png"));
    typst()?.apercu(&src, &png, page, PAGE_PPI)?;
    donnee_png(&png)
}

/// L'objet d'un envoi, tel que le canevas le manipule.
#[derive(Serialize)]
pub struct Objet {
    /// Le PNG, fond transparent, prêt à poser dans une balise `img`.
    pub image: String,
    /// Hauteur sur largeur : le canevas en a besoin pour dessiner ses prises avant que
    /// l'image ne soit chargée.
    pub ratio: f64,
}

/// L'objet d'un envoi, rendu seul sur fond transparent, avec son rapport.
///
/// Le rendre par Typst plutôt que de l'imiter en CSS fait que ce qu'on déplace **est**
/// ce qui s'imprimera : même police, même corps, mêmes coupures de lignes. La largeur
/// de rendu est celle que l'objet occupera sur la page — c'est elle qui décide des
/// coupures, et rendre à une autre largeur donnerait un rapport qui n'est pas celui du
/// tirage.
#[tauri::command]
pub fn envoi_objet(index: usize, atelier: State<Atelier>) -> Result<Objet, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let envois = &o.projet.meta.envois;
    envois.verifie()?;
    let e = envois
        .liste
        .get(index)
        .ok_or("envoi introuvable : la liste a changé.")?;
    let (pr, _, _) = vise(o)?;

    let dossier = std::env::temp_dir().join("ozalid-objet");
    std::fs::create_dir_all(&dossier)
        .map_err(|err| format!("répertoire inutilisable ({}) : {err}", dossier.display()))?;
    let src = dossier.join("objet.typ");
    let t = package::trace(&o.projet, e, &dossier)?;
    ecrire(
        &src,
        &interieur::source_objet(&t, pr.format.0 * e.place.taille),
    )?;
    let png = dossier.join("objet.png");
    // L'écriture de l'auteur vit dans le `.ozalid` : sans ce dépliage, l'objet
    // composerait dans la police de repli, et le canevas montrerait autre chose que ce
    // qui s'imprimera.
    let typst = typst()?;
    let typst = match package::ecrire_polices(&o.projet, &dossier)? {
        Some(d) => typst.avec_polices(d),
        None => typst,
    };
    typst.apercu(&src, &png, 1, OBJET_PPI)?;
    // `dimensions` rend un `Option` : un PNG que Typst vient d'écrire et qu'on ne sait
    // pas mesurer est une anomalie, pas un cas ordinaire — elle se dit plutôt que de
    // rendre un rapport inventé, qui déformerait l'objet sous la souris.
    let octets = std::fs::read(&png).map_err(|e| format!("objet illisible : {e}"))?;
    let (l, h) =
        crate::image::dimensions(&octets).ok_or("l'objet rendu n'est pas une image mesurable.")?;
    Ok(Objet {
        image: donnee_png(&png)?,
        ratio: h as f64 / l as f64,
    })
}

/// La page d'un envoi, telle qu'elle sera imprimée.
///
/// La source est celle de l'intérieur **entier**, et non plus privée de ses chapitres.
/// Le raccourci tenait tant que l'envoi se posait sur la page de titre, qui ne dépend
/// pas du corps ; depuis la v4 il vise n'importe quelle page, et la page 37 n'existe pas
/// dans un intérieur sans corps. Composer le livre complet coûte deux dixièmes de
/// seconde sur un manuscrit de 190 pages — moins que la surprise d'un aperçu qui ne
/// montre pas la bonne page.
///
/// C'est la **vérité** du canevas : celui-ci compose la page en fond et l'objet
/// séparément, puis les superpose en CSS ; ici les deux passent par Typst en une fois.
#[tauri::command]
pub fn envoi_apercu(index: usize, atelier: State<Atelier>) -> Result<String, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (pr, _, d) = vise(o)?;
    let envois = &o.projet.meta.envois;
    envois.verifie()?;
    let e = envois
        .liste
        .get(index)
        .ok_or("envoi introuvable : la liste a changé.")?;

    let int = &o.projet.meta.interieur;
    int.verifie()?;
    let livre = &o.projet.meta.livre;
    let chapitres = manuscrit::decoupe(&o.projet.texte, livre.chapitres)?;
    // La mesure du tirage, et non un réglage d'aperçu : l'aperçu montre la page que le
    // dédicataire recevra, il doit donc être composé comme elle.
    let mesure = o
        .projet
        .meta
        .livraison
        .mesure(&d.fabrication.cle_gabarit())
        .ok_or("intérieur non composé : l'aperçu a besoin des pages du tirage.")?;
    let dossier = sorties_racine(o)?.join("envois");
    std::fs::create_dir_all(&dossier)
        .map_err(|err| format!("répertoire inutilisable ({}) : {err}", dossier.display()))?;
    let src = dossier.join("apercu.typ");
    ecrire(
        &src,
        &interieur::source(
            livre,
            int,
            &pr,
            &Reglage {
                gouttiere: mesure.gouttiere,
                blanche: mesure.blanche,
            },
            &chapitres,
            Some(package::trace(&o.projet, e, &dossier)?),
        ),
    )?;
    let png = dossier.join("apercu.png");
    // L'écriture de l'auteur vit dans le `.ozalid` : sans ce dépliage, l'aperçu
    // composerait dans la police de repli, et ce serait un aperçu d'autre chose.
    let typst = typst()?;
    let typst = match package::ecrire_polices(&o.projet, &dossier)? {
        Some(d) => typst.avec_polices(d),
        None => typst,
    };
    typst.apercu(&src, &png, e.place.page, PAGE_PPI)?;
    donnee_png(&png)
}

/// Compose un package par envoi, pour le livrable visé.
///
/// Geste distinct de `packager` : l'un prépare le tirage, l'autre prépare des cadeaux,
/// et les déclencher ensemble composerait des exemplaires que personne n'a demandés.
#[tauri::command]
pub fn envoyer(atelier: State<Atelier>) -> Result<Vec<ResultatEnvoi>, String> {
    let garde = atelier.ouvert.lock().unwrap();
    let o = garde.as_ref().ok_or_else(aucun_projet)?;
    let (pr, papier, d) = vise(o)?;
    let typst = typst()?;
    let racine = sorties_racine(o)?.join("envois");

    let sorties = package::assembler_envois(&o.projet, &cible(pr, papier, d), &racine, &typst)?;

    Ok(sorties
        .into_iter()
        .zip(o.projet.meta.envois.liste.iter())
        .map(|((dossier, p), e)| ResultatEnvoi {
            dedicataire: e.dedicataire.clone(),
            dossier,
            // La vignette manquante ne perd pas le package : les PDF sont écrits.
            vignette: donnee_png(Path::new(&p.vignette)).ok(),
            package: p,
        })
        .collect())
}

/// Écrit les images du projet à côté de la source, et rend leurs descriptions.
fn ecrire_images(
    projet: &Projet,
    dossier: &Path,
) -> Result<(Option<Ressource>, Option<Ressource>), String> {
    package::ecrire_images(projet, dossier)
}

/// Racine des sorties : un répertoire du nom du projet, à côté du `.ozalid`, jamais
/// dedans. Un projet non enregistré n'a donc pas d'endroit où écrire — c'est voulu,
/// sinon les sorties atterriraient dans un répertoire temporaire que personne ne
/// retrouve. L'épreuve s'y range directement : elle ne vise aucun éditeur.
fn sorties_racine(o: &Ouvert) -> Result<PathBuf, String> {
    let chemin = o.chemin.as_ref().ok_or_else(|| {
        "enregistrer le projet avant de composer : les sorties se rangent à côté du \
         fichier .ozalid."
            .to_string()
    })?;
    let parent = chemin.parent().unwrap_or(Path::new("."));
    let nom = chemin
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "projet".into());
    Ok(parent.join(nom))
}

/// Sorties d'une clé : un répertoire par clé, sous la racine. `composer` y range le
/// gabarit d'intérieur — deux papiers du même gabarit partagent donc le répertoire de
/// travail —, `packager` y range le livrable entier.
fn sorties_dossier(o: &Ouvert, cle: &str) -> Result<PathBuf, String> {
    Ok(sorties_racine(o)?.join(cle))
}

/// Le PDF de l'intérieur d'un gabarit, là où `composer` l'écrit.
///
/// Nommé ici plutôt qu'à deux endroits : `composer` l'écrit, `vue` le cherche pour en
/// faire un lien, et deux `format!` identiques finissent par diverger — c'est la même
/// raison qui a fait servir la liste des jetons par le Rust plutôt que la recopier. Le
/// nom du fichier vient de `package::nom` ; seul l'emplacement — ce dossier précis —
/// se décide ici.
fn interieur_pdf(dossier: &Path, cle: &str) -> PathBuf {
    dossier.join(package::nom(cle, "interieur", "pdf"))
}

/// Répertoire de configuration de l'application, s'il est atteignable.
fn config(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok()
}

/// Mémorise un projet dans les récents.
///
/// **Au mieux** : un échec s'écrit sur la sortie d'erreur, visible en développement,
/// invisible pour qui lance le binaire empaqueté. C'est assumé : ce qui se perd ici
/// est une liste de raccourcis, pas un livre, et faire remonter cet échec jusqu'à
/// l'interface coûterait plus qu'il ne vaut.
fn memoriser(app: &tauri::AppHandle, chemin: &Path) {
    let Some(dir) = config(app) else {
        eprintln!("préférences : répertoire de configuration introuvable, récents non mémorisés.");
        return;
    };
    let mut p = preferences::charger(&dir);
    p.ajouter_recent(chemin);
    if let Err(e) = preferences::enregistrer(&dir, &p) {
        eprintln!("préférences : {e}");
        return;
    }
    // Le sous-menu des récents vient d'être périmé par cette écriture : le
    // reconstruire ici évite d'avoir à s'en souvenir à chaque point d'appel.
    if let Err(e) = crate::menu::poser(app) {
        eprintln!("menu : reconstruction impossible : {e}");
    }
}

fn poser(
    atelier: &State<Atelier>,
    chemin: Option<PathBuf>,
    projet: Projet,
    modifie: bool,
) -> Result<ProjetVue, String> {
    let mut garde = atelier.ouvert.lock().unwrap();
    *garde = Some(Ouvert {
        chemin,
        projet,
        modifie,
        candidat: None,
    });
    vue(garde.as_ref().unwrap())
}

/// Ce que l'écran lit de la livraison. Vue et non donnée : `compose` y est recalculée
/// par livrable depuis la mesure de son gabarit et la formule de son papier — le
/// déplacement de la mesure est invisible au front (décision du 26/08).
#[derive(Serialize)]
pub struct LivraisonVue {
    livrables: Vec<LivrableVue>,
    /// La clé du livrable visé — quatre axes.
    courant: String,
    deja_compose: bool,
}

#[derive(Serialize)]
pub struct LivrableVue {
    /// L'identité à quatre axes : l'identifiant des lignes, des DOM et des commandes.
    /// Fabriquée par le Rust et servie telle quelle — jamais recomposée côté JS : deux
    /// fabricants d'une même clé finissent par diverger.
    cle: String,
    /// La clé du gabarit : la jointure vers la liste des providers, et rien d'autre.
    gabarit: String,
    pod: String,
    format: String,
    reliure: String,
    papier: String,
    finition: Option<String>,
    dos_mm: Option<f64>,
    fond_perdu_mm: Option<f64>,
    compose: Option<MesureVue>,
}

#[derive(Serialize)]
pub struct MesureVue {
    pages: u32,
    gouttiere: f64,
    blanche: bool,
    /// Recalculé ici, jamais retenu : c'est ce qui laisse deux papiers partager une
    /// mesure, et un `dos` de formule corrigée se corriger tout seul à la vue.
    dos: Option<f64>,
    polices_introuvables: Vec<String>,
}

/// La livraison telle que le front la lit : un livrable par livrable, son identité à
/// quatre axes, et la mesure de **son gabarit** — que deux papiers partagent, chacun en
/// tirant son propre dos.
fn livraison_vue(l: &Livraison) -> LivraisonVue {
    let vue = |liv: &Livrable| -> LivrableVue {
        let f = &liv.fabrication;
        let gabarit = f.cle_gabarit();
        let compose = l.mesure(&gabarit).map(|m| {
            // Le papier du **livrable**, jamais celui d'office du POD : c'est toute la
            // différence entre deux lignes qui se comparent et deux lignes qui mentent.
            let dos = catalogue::resout(f)
                .ok()
                .and_then(|r| r.papier.dos.mm(m.pages));
            MesureVue {
                pages: m.pages,
                gouttiere: m.gouttiere,
                blanche: m.blanche,
                dos,
                polices_introuvables: m.polices_introuvables.clone(),
            }
        });
        LivrableVue {
            cle: liv.cle(),
            gabarit,
            pod: f.pod.clone(),
            format: f.format.clone(),
            reliure: f.reliure.clone(),
            papier: f.papier.clone(),
            finition: liv.finition.clone(),
            dos_mm: liv.dos_mm,
            fond_perdu_mm: liv.fond_perdu_mm,
            compose,
        }
    };
    LivraisonVue {
        livrables: l.livrables.iter().map(&vue).collect(),
        courant: l.courant.clone(),
        deja_compose: l.deja_compose,
    }
}

fn vue(o: &Ouvert) -> Result<ProjetVue, String> {
    // Le compte de chapitres affiché est celui du manuscrit embarqué, pas celui que le
    // projet déclare : c'est l'écart entre les deux qui signale un manuscrit périmé.
    let chapitres_trouves = manuscrit::decoupe(&o.projet.texte, None)
        .map(|p| p.iter().filter(|p| p.est_chapitre()).count() as u32)
        .unwrap_or(0);
    // Le lien du pied : il n'a de sens qu'avec une mesure — sans elle, le pied ne
    // montre aucun chiffre, et un PDF sans chiffres se lirait comme une pagination
    // qu'on aurait le droit de croire. `filter` et non `map` sur l'existence : le
    // fichier peut avoir été effacé à la main entre deux ouvertures.
    let interieur_pdf = o
        .projet
        .meta
        .livraison
        .courant()
        .map(|l| l.fabrication.cle_gabarit())
        .filter(|g| o.projet.meta.livraison.mesure(g).is_some())
        .and_then(|g| {
            let dossier = sorties_dossier(o, &g).ok()?;
            let pdf = interieur_pdf(&dossier, &g);
            pdf.is_file().then(|| pdf.to_string_lossy().into_owned())
        });
    Ok(ProjetVue {
        chemin: o.chemin.as_ref().map(|c| c.to_string_lossy().into_owned()),
        livre: o.projet.meta.livre.clone(),
        manuscrit_source: o.projet.meta.manuscrit.source.clone(),
        chapitres_trouves,
        mots: o.projet.texte.split_whitespace().count() as u32,
        manuscrit_absent: o.projet.texte.trim().is_empty(),
        modifie: o.modifie,
        couverture: o.projet.meta.couverture.maquette.clone(),
        couverture_importee: o.projet.meta.couverture.maquette.is_some(),
        images: o.projet.images.keys().cloned().collect(),
        interieur: o.projet.meta.interieur.clone(),
        interieur_pdf,
        livraison: livraison_vue(&o.projet.meta.livraison),
        elagues: o.projet.elagues.clone(),
        envois: o.projet.meta.envois.clone(),
    })
}

/// La vue d'un projet qu'on vient de modifier.
///
/// Deux fonctions plutôt qu'un drapeau posé à la main dans chaque commande : le
/// point d'appel dit ce qu'il a fait, et oublier de le dire se voit à la lecture.
fn vue_modifiee(o: &mut Ouvert) -> Result<ProjetVue, String> {
    o.modifie = true;
    vue(o)
}

/// La vue d'un projet qu'on vient d'écrire sur le disque.
fn vue_enregistree(o: &mut Ouvert) -> Result<ProjetVue, String> {
    o.modifie = false;
    vue(o)
}

/// Écrit le projet à un chemin, et le retient comme le sien.
///
/// Le noyau commun d'« Enregistrer » et d'« Enregistrer sous… » : les deux ne
/// diffèrent que par la façon dont le chemin est trouvé.
fn enregistrer_a(o: &mut Ouvert, chemin: &Path) -> Result<ProjetVue, String> {
    o.projet.enregistrer(chemin)?;
    o.chemin = Some(chemin.to_path_buf());
    vue_enregistree(o)
}

fn aucun_projet() -> String {
    "aucun projet ouvert.".to_string()
}

fn ecrire(chemin: &Path, contenu: &str) -> Result<(), String> {
    std::fs::write(chemin, contenu)
        .map_err(|e| format!("écriture impossible ({}) : {e}", chemin.display()))
}

/// Binaire Typst à utiliser.
///
/// En release, seul le sidecar embarqué fait foi : se rabattre sur un Typst du système
/// rendrait la pagination dépendante de la machine, exactement ce que l'embarquement
/// doit empêcher. En développement, le Typst du PATH est accepté pour ne pas imposer
/// de vendorisation à chaque itération.
fn binaire_typst() -> Result<PathBuf, String> {
    let sidecar = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join(nom_sidecar())))
        .filter(|p| p.is_file());
    match sidecar {
        Some(p) => Ok(p),
        None if cfg!(debug_assertions) => Ok(PathBuf::from("typst")),
        None => Err("Typst embarqué introuvable : l'application est mal empaquetée.".into()),
    }
}

/// Typst prêt à composer, polices embarquées comprises.
fn typst() -> Result<Typst, String> {
    let b = binaire_typst()?;
    let voisin = b.parent().map(Path::to_path_buf).unwrap_or_default();
    let candidats = [
        voisin.join("fonts"),
        // Empaquetage macOS : les ressources sont dans Contents/Resources, pas à côté
        // du binaire. Le chemin réel en release se vérifie au jalon 5.
        voisin.join("../Resources/fonts"),
        // Développement : les polices vivent dans les sources.
        Path::new(env!("CARGO_MANIFEST_DIR")).join("fonts"),
    ];
    let dossier = candidats
        .into_iter()
        .find(|p| p.is_dir())
        .ok_or("polices embarquées introuvables : lancer outils/polices.sh.")?;
    Ok(Typst::new(b).avec_polices(dossier))
}

fn nom_sidecar() -> &'static str {
    if cfg!(windows) {
        "typst.exe"
    } else {
        "typst"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Les libellés des boutons font foi au retour : le plugin rend le texte du
    /// bouton, pas un `Yes`/`No`. Une comparaison qui dériverait de l'affichage
    /// enverrait « Enregistrer » sur « ignorer », et le travail serait perdu.
    #[test]
    fn la_reponse_de_la_garde_se_lit_par_ses_libelles() {
        use tauri_plugin_dialog::MessageDialogResult as R;
        assert_eq!(reponse_garde(R::Custom(ENREGISTRER.into())), "enregistrer");
        assert_eq!(reponse_garde(R::Custom(IGNORER.into())), "ignorer");
        assert_eq!(reponse_garde(R::Custom(ANNULER.into())), "annuler");
        assert_eq!(reponse_garde(R::Yes), "enregistrer");
        assert_eq!(reponse_garde(R::No), "ignorer");
        assert_eq!(reponse_garde(R::Cancel), "annuler");
        // Fermer la boîte sans choisir ne doit rien perdre.
        assert_eq!(reponse_garde(R::Custom("autre chose".into())), "annuler");
    }

    /// L'image d'une maquette ne fait que **combler** : le livre garde ses photos, et
    /// seule une face qu'il laisse nue reçoit celle de l'archive.
    ///
    /// Sans quoi une maquette enregistrée depuis un livre imposerait sa photo à tous les
    /// autres. La règle ne regarde pas d'où vient l'archive — fournie ou personnalisée,
    /// une maquette qui porte une 1ère s'efface devant celle du livre —, et c'est de ne
    /// pas dépendre de l'origine qui la rend cohérente.
    ///
    /// La vignette montre ce que la *choisir* donnerait, pas l'archive nue, et c'est de
    /// partager [`combler_images`] avec `maquette_choisir` qui l'empêche de mentir.
    #[test]
    fn la_maquette_ne_comble_que_les_faces_que_le_livre_laisse_nues() {
        let projet = BTreeMap::from([
            ("couverture.jpg".to_string(), b"photo du livre".to_vec()),
            ("quatrieme.jpg".to_string(), b"quatrieme du livre".to_vec()),
        ]);
        let m = BTreeMap::from([
            (
                "couverture.png".to_string(),
                b"1ere de la maquette".to_vec(),
            ),
            ("quatrieme.png".to_string(), b"4eme de la maquette".to_vec()),
        ]);

        // Une maquette purement typographique : rien à poser, tout reste.
        assert_eq!(images_vignette(&projet, &BTreeMap::new()), projet);

        // Un livre déjà illustré ne bouge pas, si chargée qu'elle soit.
        assert_eq!(
            images_vignette(&projet, &m),
            projet,
            "la maquette a imposé ses photos à un livre qui avait les siennes"
        );

        // Une face nue reçoit celle de l'archive — par rôle, donc jusque sous un autre
        // nom de fichier : le livre n'a ici que sa 1ère, sous un nom à lui.
        let une_seule = BTreeMap::from([("photo.jpg".to_string(), b"photo du livre".to_vec())]);
        let f = images_vignette(&une_seule, &m);
        assert_eq!(f.get("photo.jpg"), une_seule.get("photo.jpg"));
        assert!(
            !f.contains_key("couverture.png"),
            "deux photos de 1ère se disputeraient la face"
        );
        assert_eq!(f.get("quatrieme.png"), m.get("quatrieme.png"));

        // Un livre neuf, lui, reçoit tout : c'est là que les images d'une maquette
        // servent, et la seule situation où elles se composent.
        assert_eq!(images_vignette(&BTreeMap::new(), &m), m);
    }

    fn fabrication(pod: &str, format: &str, reliure: &str, papier: &str) -> catalogue::Fabrication {
        catalogue::Fabrication {
            pod: pod.into(),
            format: format.into(),
            reliure: reliure.into(),
            papier: papier.into(),
        }
    }

    /// Tauri ne renomme que les *arguments* d'une commande, jamais les champs d'une
    /// struct : le livrable que l'interface renvoie à `livrable_regler` voyage donc en
    /// snake_case, comme le `Livre` qu'elle renvoie déjà. Le lire en camelCase ferait
    /// échouer chaque relevé de gabarit saisi, sans que rien ne dise pourquoi.
    ///
    /// Et la fabrication y est **aplatie** — `#[serde(flatten)]` : l'écran envoie les
    /// quatre axes à plat, pas un objet imbriqué. C'est le contrat du front.
    #[test]
    fn le_livrable_de_l_interface_se_lit() {
        let json = r#"{
            "pod": "kdp", "format": "6x9", "reliure": "broche", "papier": "creme",
            "dos_mm": 18.4,
            "fond_perdu_mm": 4
        }"#;
        let l: Livrable = serde_json::from_str(json).unwrap();
        assert_eq!(l.cle(), "kdp-6x9-broche-creme");
        assert_eq!(l.fabrication.cle_gabarit(), "kdp-6x9-broche");
        assert_eq!(l.dos_mm, Some(18.4));
        assert_eq!(l.fond_perdu_mm, Some(4.0));
        assert!(l.finition.is_none(), "la ligne n'en portait aucune");
    }

    /// Un relevé qu'on n'a pas encore fait est absent, pas nul : le champ vide de
    /// l'interface doit arriver ici comme une absence, faute de quoi la planche se
    /// composerait sur un dos de zéro millimètre.
    #[test]
    fn un_releve_absent_reste_absent() {
        let l: Livrable = serde_json::from_str(
            r#"{"pod": "lulu", "format": "108x175", "reliure": "broche", "papier": "standard"}"#,
        )
        .unwrap();
        assert_eq!(l.dos_mm, None);
        assert_eq!(l.fond_perdu_mm, None);
    }

    /// Le refus du doublon porte sur les quatre axes de fabrication, et sur eux seuls :
    /// deux livrables qui ne différeraient que par la finition écriraient les mêmes
    /// octets dans deux répertoires (spec § 4).
    #[test]
    fn deux_livrables_identiques_sur_les_quatre_axes_sont_refuses() {
        let un = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));
        let mut deux = un.clone();
        deux.finition = Some("mat".into());
        assert!(
            refuse_doublon(std::slice::from_ref(&un), &deux.cle()),
            "la finition a distingué deux livrables : le même fichier dans deux dossiers"
        );

        // Un axe de fabrication, lui, distingue : c'est ce qui permet de déclarer le
        // même gabarit deux fois pour comparer deux papiers.
        let blanc = Livrable::pour(fabrication("kdp", "6x9", "broche", "blanc"));
        assert!(!refuse_doublon(std::slice::from_ref(&un), &blanc.cle()));
    }

    /// Un POD qui publie une finition. Synthétique **par choix** et non par nécessité :
    /// depuis le lot 4, BoD en déclare trois et le cas « finition connue » pourrait
    /// s'ancrer sur le catalogue réel. Mais `reglage_refuse` est une règle d'application,
    /// pas un fait d'imprimeur — l'ancrer sur `bod.toml` ferait tomber ce test le jour où
    /// BoD gagne ou perd un pelliculage, pour une raison qui ne le regarde pas. Le nom
    /// « Essai » sert d'ailleurs l'assertion : c'est lui que le refus doit nommer.
    ///
    /// Sans format, reliure ni papier : `Pod::verifie` le refuserait, mais `reglage_refuse`
    /// ne lit que `nom` et `finitions`, et la fixture passe par `toml::from_str` seul.
    fn pod_a_finition() -> catalogue::Pod {
        toml::from_str(
            r#"
cle = "essai"
nom = "Essai"

[[finition]]
cle = "mat"
nom = "Pelliculage mat"
"#,
        )
        .unwrap()
    }

    /// Le POD et le format se choisissent à l'ajout, en cascade, et ne se règlent plus : les
    /// changer sur place laisserait le livrable sous une pagination qui n'est plus la
    /// sienne, et le refus dit le geste qui, lui, marche. La reliure, elle, **se règle**
    /// (spec § 6) : elle change le gabarit, le livrable retombe sur un gabarit sans mesure,
    /// et c'est exactement ce qu'une reliure exige — sa pagination admise, sa parité et sa
    /// géométrie ne sont pas celles de la précédente.
    #[test]
    fn le_pod_et_le_format_ne_se_reglent_pas_la_reliure_si() {
        let place = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));

        let autre_format = Livrable::pour(fabrication("kdp", "5x8", "broche", "creme"));
        let refus = reglage_refuse(&place, &autre_format, &pod_a_finition())
            .expect("un format changé doit être refusé");
        assert!(refus.contains("retirer"), "{refus}");

        let autre_pod = Livrable::pour(fabrication("bod", "6x9", "broche", "creme"));
        let refus = reglage_refuse(&place, &autre_pod, &pod_a_finition())
            .expect("un POD changé doit être refusé");
        assert!(refus.contains("retirer"), "{refus}");

        // La reliure se règle : c'est le geste que la spec § 6 pose sur la ligne.
        let autre_reliure = Livrable::pour(fabrication("kdp", "6x9", "rigide", "creme"));
        assert_eq!(
            reglage_refuse(&place, &autre_reliure, &pod_a_finition()),
            None,
            "la reliure doit se régler sur la ligne"
        );

        // Le papier aussi, comme depuis le lot 2.
        let autre_papier = Livrable::pour(fabrication("kdp", "6x9", "broche", "blanc"));
        assert_eq!(
            reglage_refuse(&place, &autre_papier, &pod_a_finition()),
            None
        );
    }

    /// La finition nomme une option de commande, pas un fichier : celle que le POD ne
    /// publie pas ne se commande nulle part, et la laisser passer la ferait paraître au
    /// récapitulatif comme si elle avait été retenue.
    #[test]
    fn une_finition_etrangere_au_pod_est_refusee_en_la_nommant() {
        let pod = pod_a_finition();
        let place = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));

        let mut connue = place.clone();
        connue.finition = Some("mat".into());
        assert_eq!(
            reglage_refuse(&place, &connue, &pod),
            None,
            "une finition que le POD publie doit passer"
        );

        let mut inventee = place.clone();
        inventee.finition = Some("velours".into());
        let refus = reglage_refuse(&place, &inventee, &pod).expect("finition inconnue");
        assert!(
            refus.contains("velours") && refus.contains("Essai"),
            "{refus}"
        );

        // Aucune finition n'est le cas courant : rien à vérifier, rien à refuser.
        assert_eq!(reglage_refuse(&place, &place, &pod), None);
    }

    /// La finition ne fabrique rien : elle ne change pas un octet du PDF, aucun nom de
    /// fichier ne la porte, et c'est pour ça que deux livrables ne se distinguent pas
    /// par elle. Mais elle **se commande**, et le récapitulatif est ce qu'on emporte
    /// chez l'imprimeur : muet, il fait commander un livre sans le pelliculage qu'on
    /// venait de cocher. C'est la promesse que `Livrable` écrit depuis le lot 2 — « la
    /// finition qui paraîtra au récapitulatif » —, invérifiable tant qu'aucun POD n'en
    /// déclarait, et fausse depuis que BoD en déclare trois.
    ///
    /// Le **nom**, jamais la clé : c'est « Pelliculage mat » qui se commande, pas
    /// « mat ».
    #[test]
    fn le_recapitulatif_nomme_la_finition_retenue() {
        let pod = pod_a_finition();
        let mut l = Livrable::pour(fabrication("essai", "6x9", "broche", "creme"));

        assert_eq!(
            nom_finition(&l, Some(&pod)),
            None,
            "aucune finition retenue : rien à écrire au récapitulatif"
        );

        l.finition = Some("mat".into());
        assert_eq!(
            nom_finition(&l, Some(&pod)).as_deref(),
            Some("Pelliculage mat")
        );

        // Une finition que le catalogue ne porte plus : `normalise` ne l'élague pas
        // encore, et le POD peut lui-même avoir disparu. La clé brute se lit toujours
        // — une ligne absente, elle, ne se lit pas du tout, et c'est justement le
        // silence qu'on répare ici.
        l.finition = Some("velours".into());
        assert_eq!(nom_finition(&l, Some(&pod)).as_deref(), Some("velours"));
        assert_eq!(nom_finition(&l, None).as_deref(), Some("velours"));
    }

    /// La vue que le front lit : deux papiers du même gabarit **partagent** la mesure —
    /// c'est ce qui rend la comparaison de deux papiers gratuite — et n'en tirent pas le
    /// même dos, chacun ayant sa formule.
    #[test]
    fn deux_papiers_d_un_gabarit_partagent_la_mesure_sans_partager_le_dos() {
        let creme = Livrable::pour(fabrication("kdp", "6x9", "broche", "creme"));
        let blanc = Livrable::pour(fabrication("kdp", "6x9", "broche", "blanc"));
        let empreinte = catalogue::resout(&creme.fabrication).unwrap().empreinte();
        let mut l = Livraison {
            courant: blanc.cle(),
            livrables: vec![creme, blanc],
            deja_compose: false,
            mesures: BTreeMap::new(),
        };
        l.retenir_mesure(
            "kdp-6x9-broche",
            Mesure {
                pages: 262,
                gouttiere: 25.0,
                blanche: true,
                empreinte: Some(empreinte),
                polices_introuvables: vec![],
            },
        );

        let v = livraison_vue(&l);

        assert_eq!(
            v.courant, "kdp-6x9-broche-blanc",
            "le pointeur est à quatre axes"
        );
        assert_eq!(
            v.livrables
                .iter()
                .map(|d| d.cle.as_str())
                .collect::<Vec<_>>(),
            ["kdp-6x9-broche-creme", "kdp-6x9-broche-blanc"]
        );
        let d = &v.livrables[0];
        assert_eq!(d.gabarit, "kdp-6x9-broche");
        assert_eq!(
            (
                d.pod.as_str(),
                d.format.as_str(),
                d.reliure.as_str(),
                d.papier.as_str()
            ),
            ("kdp", "6x9", "broche", "creme")
        );

        let mesure = |i: usize| {
            v.livrables[i]
                .compose
                .as_ref()
                .expect("la mesure du gabarit vaut pour ses deux papiers")
        };
        assert_eq!(mesure(0).pages, 262);
        assert_eq!(mesure(1).pages, 262);
        let (creme, blanc) = (mesure(0).dos, mesure(1).dos);
        assert!(
            creme.is_some() && blanc.is_some(),
            "KDP publie ses formules"
        );
        assert_ne!(
            creme, blanc,
            "les deux papiers rendent le même dos : le dos ne suit pas le papier du livrable"
        );
    }

    /// La vue d'arbre porte ce que la vue plate tait : les reliures d'un POD, la raison de
    /// celles qu'on n'outille pas, ses finitions. C'est elle qui alimente la cascade, et
    /// c'est le fichier qui tranche « composable » — `verifie_reliure` interdit qu'une
    /// reliure porte à la fois une géométrie et une raison de ne pas en avoir.
    #[test]
    fn la_vue_d_arbre_porte_les_reliures_avec_leur_raison() {
        let pods = pods_liste();
        let bod = pods
            .iter()
            .find(|p| p.cle == "bod")
            .expect("BoD est un POD fourni");

        assert_eq!(bod.nom, "BoD");
        assert!(
            bod.formats.iter().any(|f| f.cle == "135x215"),
            "le format de BoD manque"
        );

        let broche = bod
            .reliures
            .iter()
            .find(|r| r.cle == "broche")
            .expect("BoD brochera toujours");
        assert!(
            broche.non_outille.is_none(),
            "le broché est composable : aucune raison à afficher"
        );

        let rigide = bod
            .reliures
            .iter()
            .find(|r| r.cle == "rigide")
            .expect("BoD publie une couverture rigide qu'on n'outille pas");
        let raison = rigide
            .non_outille
            .as_deref()
            .expect("une reliure non outillée dit pourquoi");
        // Sur ce qui manque, et non sur le nom de la reliure : ce nom est déjà à côté
        // dans l'interface — « Couverture rigide — <raison> » — et l'y redire était
        // justement ce que la raison faisait de trop. Un test qui vérifie une redite
        // empêche de la retirer.
        assert!(
            raison.contains("ni rempli, ni mors, ni cartons"),
            "{raison}"
        );
    }

    /// Le drapeau voyage jusqu'à l'arbre et y suit le fichier, papier par papier.
    ///
    /// Depuis le lot 5, **les six PODs fournis publient une formule pour chacun de leurs
    /// papiers** : le dernier qui ne le faisait pas, CoolLibri, a vu son calculateur
    /// relevé. Plus un seul dos ne se saisit à la main dans le catalogue livré, et c'est
    /// ce que ce test affirme — un POD fourni qui reviendrait au relevé le ferait rougir,
    /// et ce serait la bonne conversation à avoir.
    ///
    /// La forme `mesure` reste servie pour un fichier déposé sur le poste ; c'est
    /// `la_conversion_d_un_papier_suit_sa_propre_formule_de_dos` qui couvre la règle
    /// « par papier, pas par POD » sur une fixture construite pour l'exercer.
    #[test]
    fn dos_publie_est_porte_par_chaque_papier() {
        for pod in pods_liste() {
            assert!(
                pod.papiers.iter().all(|pa| pa.dos_publie),
                "{} : tout papier fourni porte sa formule de dos",
                pod.cle
            );
            assert!(!pod.papiers.is_empty(), "{} sans papier", pod.cle);
        }
    }

    /// Le relevé de dos suit le **papier**, jamais le premier de la liste. Aucun POD fourni
    /// ne le vérifie : KDP publie une formule pour ses deux papiers, CoolLibri pour aucun —
    /// la mutation qui calculerait `dos_publie` sur le papier d'office du POD plutôt que sur
    /// chaque papier resterait donc invisible à `dos_publie_est_porte_par_chaque_papier`.
    /// Cette fixture mélange exprès les deux formes dans le même POD pour que la règle
    /// rougisse, et passe par `PodVue::from` — le site d'appel réel, celui qu'une régression
    /// toucherait — plutôt que par `PapierVue::from` en direct, qui ne voit pas le POD.
    #[test]
    fn la_conversion_d_un_papier_suit_sa_propre_formule_de_dos() {
        let pod = catalogue::Pod::depuis_toml(
            r##"
cle = "essai"
nom = "Imprimeur d'essai"

[[format]]
cle = "135x215"
nom = "13,5 × 21,5 cm"
mm = { largeur = 135.0, hauteur = 215.0 }
marges = { haut = 18.8, bas = 28.0, exterieur = 15.0 }
gouttieres = [{ de = 24, a = 900, mm = 20.0 }]

[[reliure]]
cle = "broche"
nom = "Broché — dos carré collé"
geometrie = "dos-carre-colle"
pages = { min = 24, max = 900 }
parite = "paire"

[[papier]]
cle = "mesure"
nom = "Dos relevé sur le gabarit"
teinte = "#ffffff"
dos = { forme = "mesure" }

[[papier]]
cle = "creme-90"
nom = "Crème 90 g"
teinte = "#f7f0e0"
dos = { forme = "multiplie", par = 0.0675, plus = 0.6 }
"##,
        )
        .unwrap();

        let vue = PodVue::from(&pod);
        assert!(
            !vue.papiers[0].dos_publie,
            "« mesure » ne publie aucune formule"
        );
        assert!(vue.papiers[1].dos_publie, "« multiplie » en publie une");
    }

    /// Choisir l'image d'une face remplace celle qui s'y composait, quel que soit le
    /// nom qu'elle portait — un projet importé nomme ses photos comme il l'entend — et
    /// laisse l'autre face intacte.
    #[test]
    fn une_face_ne_garde_qu_une_image() {
        let mut images = BTreeMap::from([
            ("photo.jpg".to_string(), vec![1]),
            ("quatrieme.jpg".to_string(), vec![2]),
        ]);

        poser_image(&mut images, "couverture.png".into(), vec![3]);
        assert_eq!(
            images.keys().collect::<Vec<_>>(),
            ["couverture.png", "quatrieme.jpg"],
            "l'image de 1ère n'a pas été remplacée, ou la 4ème a été emportée"
        );

        poser_image(&mut images, "quatrieme.png".into(), vec![4]);
        assert_eq!(
            images.keys().collect::<Vec<_>>(),
            ["couverture.png", "quatrieme.png"]
        );
    }

    /// Retirer une photo ne retire que celle-là, et un nom que le projet ne porte pas
    /// est refusé.
    ///
    /// Le refus n'est pas une politesse : la fenêtre clique un nom que cette vue-là
    /// venait de lui servir. Qu'il ait disparu entre les deux dit une liste périmée,
    /// donc un geste qui a porté sur autre chose que ce qu'on voyait — et réussir en
    /// silence laisserait croire la photo partie alors qu'une autre est restée.
    #[test]
    fn retirer_une_photo_ne_touche_que_celle_la() {
        let mut images = BTreeMap::from([
            ("couverture.jpeg".to_string(), vec![1]),
            ("quatrieme.jpeg".to_string(), vec![2]),
        ]);

        retirer_image(&mut images, "quatrieme.jpeg").unwrap();
        assert_eq!(
            images.keys().collect::<Vec<_>>(),
            ["couverture.jpeg"],
            "la 1ère est partie avec la 4ème, ou la 4ème est restée"
        );

        let refus = retirer_image(&mut images, "quatrieme.jpeg").unwrap_err();
        assert!(refus.contains("quatrieme.jpeg"), "{refus}");
    }

    /// Le nom porte le rôle : c'est tout ce que la composition lit pour savoir quelle
    /// face une image sert.
    #[test]
    fn le_nom_d_une_image_dit_la_face_qu_elle_sert() {
        assert_eq!(nom_image("une", "jpg").unwrap(), "couverture.jpg");
        assert_eq!(nom_image("quatre", "png").unwrap(), "quatrieme.png");
        assert!(package::sert_la_quatrieme(
            &nom_image("quatre", "png").unwrap()
        ));
        assert!(!package::sert_la_quatrieme(
            &nom_image("une", "png").unwrap()
        ));
        assert!(nom_image("planche", "png").is_err());
    }

    /// La clé du modèle est en clair dans `preferences.toml`, avec les permissions du
    /// fichier : c'est un choix, et il ne tient que si elle ne va nulle part ailleurs.
    /// Ce test tombe le jour où quelqu'un ajoute le champ à la vue — c'est-à-dire le
    /// jour où la clé entrerait dans une page, donc dans une capture d'écran.
    #[test]
    fn la_vue_de_l_acces_au_modele_ne_porte_pas_la_cle() {
        let v = AccesVue {
            url: "https://exemple.test/images".into(),
            modele: "gemini-3-pro-image".into(),
            cle_posee: true,
        };
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json.matches("cle").count(), 1, "un second « cle » : {json}");
        assert!(json.contains("cle_posee"), "{json}");
    }

    fn ouvert_neuf() -> Ouvert {
        Ouvert {
            chemin: None,
            projet: Projet::nouveau(Livre::vide(), String::new()),
            modifie: false,
            candidat: None,
        }
    }

    /// Le drapeau est ce qui décide si fermer l'application perd du travail. Il ne
    /// doit se lever que par une mutation, et retomber par une écriture — jamais
    /// par une simple relecture du projet.
    #[test]
    fn le_drapeau_de_modification_suit_les_mutations_et_les_ecritures() {
        let mut o = ouvert_neuf();
        assert!(
            !vue(&o).unwrap().modifie,
            "un projet neuf n'est pas modifié"
        );
        assert!(!vue(&o).unwrap().modifie, "relire ne modifie pas");

        assert!(vue_modifiee(&mut o).unwrap().modifie);
        assert!(vue(&o).unwrap().modifie, "le drapeau reste levé");

        assert!(!vue_enregistree(&mut o).unwrap().modifie);
    }

    /// Un manuscrit absent et un manuscrit sans chapitre composable rendent tous
    /// deux zéro chapitre. L'interface doit pouvoir dire « aucun manuscrit » plutôt
    /// que « 0 chapitre » : ce n'est pas la même chose à corriger.
    #[test]
    fn un_manuscrit_vide_se_declare_absent_et_non_vide_de_chapitres() {
        let vide = ouvert_neuf();
        let v = vue(&vide).unwrap();
        assert!(v.manuscrit_absent);
        assert_eq!(v.chapitres_trouves, 0);

        let mut plein = ouvert_neuf();
        plein.projet.texte = "## 01 - Un\n\nTexte.\n".into();
        let v = vue(&plein).unwrap();
        assert!(!v.manuscrit_absent);
        assert_eq!(v.chapitres_trouves, 1);

        // Du texte qui ne porte aucun « ## » : présent, mais sans chapitre.
        let mut sans_chapitre = ouvert_neuf();
        sans_chapitre.projet.texte = "juste une phrase\n".into();
        let v = vue(&sans_chapitre).unwrap();
        assert!(!v.manuscrit_absent, "présent, même s'il ne compose pas");
        assert_eq!(v.chapitres_trouves, 0);

        // Des espaces et des sauts de ligne ne sont pas un manuscrit : c'est ce que
        // `trim` établit, et rien ne le dirait si on le retirait.
        let mut blancs = ouvert_neuf();
        blancs.projet.texte = "  \n\n\t \n".into();
        assert!(vue(&blancs).unwrap().manuscrit_absent);
    }

    /// Écrire, c'est aussi retenir où : un « Enregistrer » suivant doit réécrire au
    /// même endroit sans rien redemander.
    #[test]
    fn enregistrer_retient_le_chemin_ecrit() {
        let dir = tempfile::tempdir().unwrap();
        let chemin = dir.path().join("livre.ozalid");
        let mut o = ouvert_neuf();
        o.modifie = true;

        let v = enregistrer_a(&mut o, &chemin).unwrap();
        assert!(!v.modifie, "le drapeau retombe à l'écriture");
        assert_eq!(o.chemin.as_deref(), Some(chemin.as_path()));
        assert!(chemin.is_file(), "l'archive est bien sur le disque");
        // Relire, et pas seulement écrire : un projet neuf porte un manuscrit vide,
        // et l'archive doit tout de même le contenir pour être relisible.
        assert_eq!(Projet::ouvrir(&chemin).unwrap().texte, "");
    }

    /// Une écriture refusée ne doit ni faire retomber le drapeau, ni faire croire que
    /// le projet a changé d'adresse. C'est le cas où l'on croirait avoir sauvegardé.
    #[test]
    fn une_ecriture_refusee_ne_deplace_ni_le_projet_ni_le_drapeau() {
        let dir = tempfile::tempdir().unwrap();
        // Un répertoire existant ne peut pas être ouvert en création de fichier :
        // c'est un échec d'écriture qui n'exige ni permission ni disque plein.
        let impossible = dir.path().join("sous-repertoire");
        std::fs::create_dir(&impossible).unwrap();

        let ancien = dir.path().join("ancien.ozalid");
        let mut o = ouvert_neuf();
        o.modifie = true;
        o.chemin = Some(ancien.clone());

        assert!(enregistrer_a(&mut o, &impossible).is_err());
        assert!(o.modifie, "le drapeau reste levé");
        assert_eq!(o.chemin.as_deref(), Some(ancien.as_path()));
    }

    /// Le genre par défaut ne doit vivre qu'à un endroit : un projet neuf et un
    /// projet relu d'un TOML sans genre doivent porter le même.
    #[test]
    fn un_livre_vide_prend_le_genre_par_defaut() {
        let l = Livre::vide();
        assert_eq!(l.genre, "Genre");
        assert_eq!(l.titre, "Titre");
        assert_eq!(l.auteur, "Auteur");
        assert_eq!(l.chapitres, None);
        assert_eq!(l.titre_page, "%TITRE%");
    }

    /// Un projet neuf part avec le gabarit que l'utilisateur a posé pour défaut : c'est
    /// tout l'objet du réglage — ne pas le retaper d'un livre à l'autre.
    #[test]
    fn un_projet_neuf_recoit_le_gabarit_de_depart() {
        let p = projet_neuf("une aquarelle pour {dedicataire}".into());
        assert_eq!(p.meta.envois.gabarit, "une aquarelle pour {dedicataire}");
    }

    /// Le défaut n'est qu'une valeur de départ : il ne touche ni la liste des envois, ni
    /// la couleur, ni le paraphe, qui appartiennent au livre qu'on écrit.
    #[test]
    fn le_gabarit_de_depart_ne_pose_rien_d_autre() {
        let p = projet_neuf("une aquarelle".into());
        assert!(p.meta.envois.liste.is_empty());
        assert_eq!(p.meta.envois.couleur, "");
        assert_eq!(p.meta.envois.paraphe, "");
    }
}
