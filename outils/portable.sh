#!/usr/bin/env bash
# Assemble l'archive portable pour Windows.
#
# Tauri 2 n'offre sous Windows que les cibles `nsis` et `msi` : il n'y a pas de cible
# « portable » à demander à `tauri build`, l'archive se monte à la main. Ici et non dans
# le workflow, pour la même raison que `typst.sh` et `polices.sh` : ce qui n'existe que
# dans la CI ne se vérifie que dans la CI, et personne n'ouvrirait le portable avant le
# premier utilisateur.
set -euo pipefail

ICI="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TRIPLE="x86_64-pc-windows-msvc"
EXE="$ICI/src-tauri/target/release/ozalid-studio.exe"
SIDECAR="$ICI/src-tauri/binaries/typst-${TRIPLE}.exe"
POLICES="$ICI/src-tauri/fonts"
SORTIE="$ICI/src-tauri/target/portable"

for f in "$EXE" "$SIDECAR"; do
  [ -f "$f" ] || { echo "absent : $f" >&2; exit 1; }
done
ls "$POLICES"/*.ttf >/dev/null 2>&1 || { echo "aucune police dans $POLICES" >&2; exit 1; }

# La même source de vérité que le contrôle de tag du job `publier` : la version de
# l'application est celle de tauri.conf.json, et rien d'autre ne la porte.
# En relatif et non en absolu : dans le Git Bash des runners Windows, `$ICI` est un
# chemin MSYS (`/d/a/...`) que le runtime ne convertit que pour un argument qui
# ressemble à un chemin — pas celui-ci, qui commence par `require(`. Node, binaire
# natif, le résoudrait alors depuis la racine du lecteur courant : `MODULE_NOT_FOUND`.
# `.github/workflows/windows.yml` fait déjà ce `node -p` en relatif pour la même raison.
VERSION="$(cd "$ICI" && node -p "require('./src-tauri/tauri.conf.json').version")"
NOM="Ozalid Studio $VERSION"

rm -rf "$SORTIE"
mkdir -p "$SORTIE/$NOM"
cp "$EXE" "$SORTIE/$NOM/ozalid-studio.exe"
# Renommé, comme le bundle Tauri le fait d'un `externalBin` : `commands.rs::binaire_typst`
# cherche « typst.exe » à côté de l'exécutable, pas le nom triplé.
cp "$SIDECAR" "$SORTIE/$NOM/typst.exe"
mkdir -p "$SORTIE/$NOM/fonts"
cp "$POLICES"/* "$SORTIE/$NOM/fonts/"
# Le marqueur : vide, nommé d'après l'exécutable. C'est lui, et lui seul, qui fait
# écrire les réglages dans le « donnees/ » voisin plutôt que sur la machine hôte.
# `donnees/` n'est pas livré : il naît au premier lancement, et une archive qui le
# contiendrait ferait croire à un dossier qu'on peut écraser en mettant à jour.
: > "$SORTIE/$NOM/ozalid-studio.portable"

ARCHIVE="$SORTIE/ozalid-studio_${VERSION}_x64-portable.zip"

# Écrire un zip demande un archiveur qui sache en écrire, et « tar » n'est pas le même
# programme partout. Sur macOS c'est bsdtar, qui le sait. Dans le Git Bash des runners
# Windows c'est GNU tar, qui ne le sait pas : il écrit un *tar* nommé `.zip`, et le
# constat s'est fait en release (run 33409871203). Windows fournit pourtant bsdtar
# depuis la 1803, sous System32 — mais le tar de Git Bash le masque dans le PATH. On va
# donc le chercher là où il est, avec `Compress-Archive` en dernier recours : PowerShell
# est présent partout où Git Bash tourne.
#
# `zip` n'est candidat nulle part : il n'est pas garanti dans ce Git Bash, la même raison
# qui fait passer `typst.sh` par `tar` pour l'extraction.
zippe() {
  local candidat
  for candidat in tar /c/Windows/System32/tar.exe; do
    if "$candidat" --version 2>/dev/null | grep -qi bsdtar; then
      (cd "$SORTIE" && "$candidat" -a -cf "$ARCHIVE" "$NOM") && return 0
    fi
  done
  if command -v cygpath >/dev/null 2>&1 && command -v powershell >/dev/null 2>&1; then
    powershell -NoProfile -Command \
      "Compress-Archive -Path '$(cygpath -w "$SORTIE/$NOM" 2>/dev/null)' -DestinationPath '$(cygpath -w "$ARCHIVE" 2>/dev/null)' -Force" \
      && return 0
  fi
  return 1
}

if ! zippe; then
  # Le diagnostic sur place : sans lui, un poste où aucun candidat ne convient rendrait
  # le même message que le précédent échec sans dire lequel a été essayé.
  echo "aucun archiveur capable d'écrire un zip sur ce poste." >&2
  echo "  tar du PATH  : $(tar --version 2>&1 | head -1)" >&2
  echo "  System32     : $(/c/Windows/System32/tar.exe --version 2>&1 | head -1)" >&2
  echo "  powershell   : $(command -v powershell || echo absent)" >&2
  exit 1
fi

# Une archive zip commence par la signature « PK ». Le vérifier ici garde l'ambiguïté
# bruyante à la source, quel que soit le candidat retenu : c'est ce contrôle, et non le
# dépliage côté CI, qui a nommé le GNU tar de Git Bash.
[ "$(head -c2 "$ARCHIVE")" = "PK" ] || { echo "l'archive produite n'est pas un zip malgré $(tar --version 2>&1 | head -1)" >&2; exit 1; }

echo "$ARCHIVE"
