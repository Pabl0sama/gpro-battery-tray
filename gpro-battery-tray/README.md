# gpro-battery-tray

Petit utilitaire Windows qui affiche le **niveau de batterie d'une souris Logitech G Pro Wireless** directement dans la zone de notification (à côté de l'horloge).

- Pourcentage affiché dans l'icône, coloré selon le niveau (vert, orange, rouge)
- Barre bleue sous le chiffre quand la souris est en charge
- Info-bulle au survol avec le détail
- Mise à jour automatique toutes les 60 secondes, ou à la demande via le menu clic droit
- Un seul `.exe` léger, sans console, sans dépendance à installer
- Fonctionne en parallèle de Logitech G Hub (G Hub peut rester ouvert)

## Compatibilité

| | |
|---|---|
| Système | Windows 10 / 11 |
| Testé avec | Logitech G Pro Wireless + récepteur Lightspeed (USB `046D:C539`) |
| Autres souris Logitech | Probablement compatibles si elles utilisent HID++ 2.0 avec les fonctionnalités batterie `0x1000`, `0x1001` ou `0x1004`, mais non testées |

## Installation

### 1. Installer Rust

1. Télécharge et lance `rustup-init.exe` depuis <https://rustup.rs>.
2. Si l'installateur signale que les **Visual Studio C++ Build Tools** sont manquants, accepte leur installation (ou installe-les depuis <https://visualstudio.microsoft.com/visual-cpp-build-tools/> en cochant « Développement Desktop en C++ »).
3. Choisis l'installation par défaut (option `1`).
4. **Ferme puis rouvre PowerShell**, puis vérifie :
   ```powershell
   cargo --version
   ```

### 2. Récupérer le projet

```powershell
git clone https://github.com/TON_PSEUDO/gpro-battery-tray.git
cd gpro-battery-tray
```

Sans Git, télécharge l'archive ZIP depuis la page GitHub (bouton **Code**, puis **Download ZIP**) et décompresse-la.

### 3. Compiler

```powershell
cargo build --release
```

La première compilation télécharge les dépendances et peut durer quelques minutes.
L'exécutable est généré ici :

```
target\release\gpro-battery-tray.exe
```

### 4. Lancer

Double-clique sur `gpro-battery-tray.exe`. L'icône apparaît dans la zone de notification. Si elle n'est pas visible, regarde dans le menu des icônes cachées (`^`) et glisse-la vers la barre des tâches pour l'épingler.

> Avant de recompiler une nouvelle version, quitte l'application (clic droit sur l'icône, puis **Quitter**), sinon Windows empêche de remplacer l'`.exe`.

## Lancement automatique au démarrage de Windows

1. Copie `gpro-battery-tray.exe` dans un dossier fixe, par exemple `C:\Users\TON_NOM\Outils\`.
2. Clic droit sur l'`.exe`, puis **Afficher plus d'options**, puis **Créer un raccourci**.
3. Appuie sur `Win + R`, tape `shell:startup` et valide.
4. Déplace le raccourci dans le dossier qui s'ouvre.

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

**L'icône affiche `?` ou « introuvable »**

- Vérifie que le récepteur USB est branché directement sur le PC (évite les hubs).
- Réveille la souris (bouge-la ou clique), puis clique sur **Actualiser**. Une souris en veille ne répond pas.
- Si le problème persiste, lance l'outil de diagnostic et joins sa sortie dans une *issue* :
  ```powershell
  cargo run --release --bin diag
  ```
  Il liste les interfaces HID Logitech détectées et teste les réponses de la souris. L'interface utile est celle avec `usage_page FF00` et `usage 0002` ; les erreurs `Fonction incorrecte` sur les autres interfaces sont normales.

**`cargo` n'est pas reconnu**

Rouvre PowerShell après l'installation de Rust. Sinon, vérifie `Test-Path "$env:USERPROFILE\.cargo\bin\cargo.exe"` et redémarre le PC si le résultat est `True`.

**Erreur de compilation sur une dépendance**

Les versions de `tray-icon`, `tao` et `hidapi` évoluent. Ouvre une *issue* en collant le message d'erreur complet.

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

## Structure du projet

```
gpro-battery-tray/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs        # application (icône + lecture batterie)
    └── bin/
        └── diag.rs    # outil de diagnostic HID++
```

## Avertissement

Projet indépendant, non affilié à Logitech. Le protocole HID++ est documenté de manière non officielle, notamment par le projet [Solaar](https://github.com/pwr-Solaar/Solaar). Utilisation à tes propres risques.

## Licence

À choisir (par exemple MIT). Ajoute un fichier `LICENSE` à la racine du dépôt.
