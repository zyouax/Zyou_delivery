# Zyou_delivery

<p align="center">
  <img src=".github/Zyou_delivery.png" height="300" alt="Zyou_delivery Logo">
</p>

<h1 align="center">Zyou_delivery 🚚</h1>

<p align="center">
  <a href="https://www.rust-lang.org/" title="Go to Rust homepage"><img src="https://img.shields.io/badge/Rust-1-blue?logo=rust&logoColor=white" alt="Made with Rust"></a>
  <a href="https://www.rust-lang.org/" title="Go to Rust homepage"><img src="https://img.shields.io/badge/Crate-Zyou_Delivery-green?logo=crate&logoColor=black" alt="Made with Rust"></a>
  <a href="https://github.com/zyouax/Zyou_delivery/actions"><img src="https://img.shields.io/github/workflow/status/zyouax/Zyou_delivery/CI?label=Tests&style=flat-square" alt="Tests"></a>
  <a href="https://github.com/zyouax/Zyou_delivery/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue?style=flat-square" alt="License"></a>
  <a href="https://github.com/zyouax/Zyou_delivery"><img src="https://img.shields.io/github/stars/zyouax/Zyou_delivery?style=flat-square" alt="GitHub Stars"></a>
</p>

<p align="center">
  <strong>Une bibliothèque Rust pour gérer vos expéditions avec simplicité !</strong><br>
  Intégrez facilement les APIs de transporteurs comme <em>Colissimo</em>, <em>FedEx</em>, <em>Chronopost</em>, <em>TNT</em>, <em>UPS</em> et <em>Mondial Relay</em>.
</p>

---

### Annexe

- [Installation](#-installation), Installation via Crate.io ou GitHub
- [Utilisation](#-utilisation), Importer et utiliser la bibliothèque
- [Bon à savoir](#bon-à-savoir), Comment activer les transporteurs

## ✨ Pourquoi Zyou_delivery ?

`Zyou_delivery` simplifie la logistique e-commerce :
- **Une seule API** pour tous les transporteurs.
- **Fiable** : Gestion des erreurs et rétentatives automatiques.
- **Facile à utiliser** : Exemples clairs et code modulaire.
- **Tests robustes** : Simulation des APIs avec `mockito` pour des tests rapides.
- **Flexible** : Activez uniquement les transporteurs nécessaires.

## 🚀 Fonctionnalités

- 📊 **Tarifs** : Récupérez les coûts d'expédition en fonction du colis.
- 🏷️ **Étiquettes** : Générez des étiquettes avec numéros de suivi.
- 📍 **Suivi** : Suivez vos colis en temps réel.
- 🌍 **Transporteurs** : Colissimo, FedEx, Chronopost, TNT, UPS, Mondial Relay.
- 🔄 **Formats** : Supporte JSON et XML selon les APIs.

---

## 🛠️ Installation

### Prérequis
- Rust 1.65+ (`rustc --version`)
- Clés API des transporteurs
- Fichier `.env` pour les configurations

### Configurer avec Crate.io ou avec GitHub
1. **Crate.io** : Ajoutez la dépendance à votre fichier `Cargo.toml` :
   ```toml
   [dependencies]
   zyou_delivery = "0.1.0" [features = ["colissimo", "fedex", "ups"]]
   ```
2. 1. **GitHub** : Clonez le dépôt :
   ```bash
   git clone https://github.com/zyouax/Zyou_delivery.git
   cd Zyou_delivery
   ```
   2. **Terminal** : Copier le fichier `.env.tests` en `.env` :
   ```bash
   cp .env.tests .env
   ```
   ## 2 Options Au choix
   1. **Terminal** : Compiler le projet (avec tous les transporteurs) :
   ```bash
   cargo build --features all
   ```
   2. **Terminal** : Compiler le projet (avec seulement les transporteurs specificés) :
   ```bash
   cargo build --features colissimo,fedex,ups,tnt,chronopost,mondialrelay
   ```

### Bon à savoir

- **Seul le transporteur Colissimo est Activés par défaut.** Pour activer les autres, ajoutez les noms des transporteurs dans le tableau `[features]` de votre fichier `Cargo.toml`.

> Vous pouvez utiliser ```[features = "all"]``` pour activer tous les transporteurs.

   > Vous pouvez activer les transporteurs individuellement avec les noms de features suivants :
   > - `colissimo`
   > - `fedex`
   > - `ups`
   > - `tnt`
   > - `chronopost`
   > - `mondialrelay`

## 📝 Utilisation
