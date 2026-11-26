# gpro-battery-tray

Petit utilitaire Windows qui affiche le **niveau de batterie d'une souris Logitech G Pro Wireless** directement dans la zone de notification (à côté de l'horloge).

- Pourcentage affiché dans l'icône, coloré selon le niveau (vert, orange, rouge)
- Barre bleue sous le chiffre quand la souris est en charge
- Info-bulle au survol avec le détail
- Mise à jour automatique toutes les 60 secondes, ou à la demande via le menu clic droit
- Un seul `.exe` léger, sans console
- Fonctionne en parallèle de Logitech G Hub (G Hub peut rester ouvert)

## Compatibilité

| | |
|---|---|
| Système | Windows 10 / 11 |
| Testé avec | Logitech G Pro Wireless + récepteur Lightspeed (USB `046D:C539`) fonctionnel |
| Testé avec | Attack Shark X3 + récepteur USB non fonctionnel |
| Autres souris | Probablement compatibles si elles utilisent HID++ 2.0 avec les fonctionnalités batterie `0x1000`, `0x1001` ou `0x1004`, mais non testées |

## Ce dépôt contient les sources

Il n'y a pas d'exécutable prêt à l'emploi dans les fichiers du dépôt : il faut **compiler le programme** une fois sur ton PC. Le tutoriel ci-dessous explique comment faire, étape par étape. La compilation prend quelques minutes.

```
gpro-battery-tray/
├── Cargo.toml         # configuration du projet et dépendances
├── README.md
├── .gitignore
└── src/
    ├── main.rs        # application (icône + lecture de la batterie)
    └── bin/
        └── diag.rs    # outil de diagnostic HID++
```

## Tutoriel de compilation

### Étape 1 : installer Rust

1. Télécharge `rustup-init.exe` depuis <https://rustup.rs> et lance-le.
2. Si l'installateur signale que les **Visual Studio C++ Build Tools** sont manquants, accepte leur installation. Tu peux aussi les installer à part depuis <https://visualstudio.microsoft.com/visual-cpp-build-tools/> en cochant **Développement Desktop en C++**.
3. Choisis l'installation par défaut (tape `1` puis Entrée).
4. **Ferme toutes les fenêtres PowerShell, puis ouvres-en une nouvelle.** C'est indispensable, sinon la commande `cargo` ne sera pas reconnue.
5. Vérifie l'installation :
   ```powershell
   cargo --version
   ```
   Une ligne du type `cargo 1.xx.x` doit s'afficher.
   /!\ Nécessite peut-être un redémarrage du PC. /!\

### Étape 2 : récupérer les sources

**Avec Git :**
```powershell
git clone https://github.com/Pabl0sama/gpro-battery-tray.git
cd gpro-battery-tray
```

**Sans Git :** sur la page GitHub du projet, clique sur **Code**, puis **Download ZIP**. Décompresse l'archive, ouvre PowerShell dans le dossier obtenu (clic droit dans le dossier en maintenant `Maj`, puis **Ouvrir la fenêtre PowerShell ici**), ou utilise `cd` :
```powershell
cd C:\chemin\vers\gpro-battery-tray
```

Vérifie que tu es au bon endroit : la commande `dir` doit afficher `Cargo.toml` et le dossier `src`.

### Étape 3 : compiler

```powershell
cargo build --release
```

La première compilation télécharge les dépendances et peut durer plusieurs minutes. À la fin, tu dois voir une ligne `Finished release`.

### Étape 4 : récupérer l'exécutable

Le programme se trouve ici :

```
target\release\gpro-battery-tray.exe
```

Le dossier `target` n'existe qu'après la compilation. Copie l'`.exe` dans un dossier fixe, par exemple `C:\Users\TON_NOM\Outils\`.

### Étape 5 : lancer

Double-clique sur `gpro-battery-tray.exe`. L'icône apparaît dans la zone de notification. Si tu ne la vois pas, regarde dans le menu des icônes cachées (`^`) et glisse-la vers la barre des tâches pour l'épingler.

> Avant de recompiler une nouvelle version, quitte l'application (clic droit sur l'icône, puis **Quitter**). Sinon Windows empêche de remplacer l'`.exe`.

### Étape 6 (facultative) : lancement automatique au démarrage

1. Clic droit sur `gpro-battery-tray.exe`, puis **Afficher plus d'options**, puis **Créer un raccourci**.
2. Appuie sur `Win + R`, tape `shell:startup` et valide.
3. Déplace le raccourci dans le dossier qui s'ouvre.

## Utilisation

| Élément | Signification |
|---|---|
| Chiffre vert | Batterie supérieure à 30 % |
| Chiffre orange | Batterie entre 16 % et 30 % |
| Chiffre rouge | Batterie de 15 % ou moins |
| Barre bleue | Souris en charge |
| `?` gris | Souris non trouvée |
| Clic droit, **Actualiser** | Force une lecture immédiate |
| Clic droit, **Quitter** | Ferme l'application |

## Dépannage

### Problèmes de compilation

| Message | Solution |
|---|---|
| `cargo` n'est pas reconnu | Ferme et rouvre PowerShell. Sinon, vérifie `Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"` : si le résultat est `True`, redémarre le PC. |
| `could not find Cargo.toml` | Tu n'es pas dans le bon dossier. Utilise `cd` vers le dossier qui contient `Cargo.toml`. |
| `linker 'link.exe' not found` | Les Visual Studio C++ Build Tools ne sont pas installés. Installe-les (voir étape 1), puis relance `cargo build --release`. |
| Erreur sur `tray-icon`, `tao` ou `hidapi` | Les versions de ces bibliothèques évoluent. Ouvre une *issue* en collant le message d'erreur complet. |
| `Accès refusé` en écrasant l'`.exe` | Quitte l'application via le menu de l'icône, puis recompile. |

### Problèmes d'exécution

**L'icône affiche `?` ou « introuvable »**

- Vérifie que le récepteur USB est branché directement sur le PC (évite les hubs).
- Réveille la souris (bouge-la ou clique), puis clique sur **Actualiser**. Une souris en veille ne répond pas.
- Si le problème persiste, lance l'outil de diagnostic et joins sa sortie dans une *issue* :
  ```powershell
  cargo run --release --bin diag
  ```
  Il liste les interfaces HID Logitech détectées et teste les réponses de la souris. L'interface utile est celle avec `usage_page FF00` et `usage 0002` ; les erreurs `Fonction incorrecte` sur les autres interfaces sont normales.

## Fonctionnement

1. Le récepteur Lightspeed expose une interface HID « constructeur » (`usage_page 0xFF00`, `usage 0x0002`) qui accepte les messages **HID++ 2.0** de 20 octets (rapport `0x11`).
2. Le programme interroge la fonctionnalité `IRoot` (`getFeature`) pour trouver l'index de la fonctionnalité batterie parmi, dans l'ordre : `0x1004` (pourcentage), `0x1000` (niveau), `0x1001` (tension).
3. La G Pro Wireless fournit une **tension** en millivolts. Elle est convertie en pourcentage par interpolation dans la table `VOLTAGE_TABLE` de `src/main.rs`.
4. L'icône est dessinée pixel par pixel avec une petite police bitmap intégrée, puis affichée via `tray-icon`.
5. Un thread secondaire interroge la souris ; le thread principal gère l'icône et le menu.

## Limites connues

- Le pourcentage de la G Pro Wireless est une **estimation** basée sur la tension (précision de l'ordre de ±5 %). Tu peux ajuster `VOLTAGE_TABLE` si les valeurs te semblent décalées.
- Si la souris est en veille, la dernière valeur connue reste affichée.
- Un seul périphérique est affiché : le premier qui répond sur le récepteur.

## Avertissement

Projet indépendant, non affilié à Logitech. Le protocole HID++ est documenté de manière non officielle, notamment par le projet [Solaar](https://github.com/pwr-Solaar/Solaar). Utilisation à tes propres risques.

## Licence

À choisir (par exemple MIT). Ajoute un fichier `LICENSE` à la racine du dépôt.
