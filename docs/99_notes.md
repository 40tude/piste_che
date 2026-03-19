# Development Notes


## TODO

Penser à **CRÉER DES ISSUES** sur GitHub

- [x] Passer tous les tests
- [x] Piste 6 pixels partout et pas par segment
- [x] Problème aux points d'arrivée et de départ qui sont pas toujours les plus proches sur les piste
- [x] Vérifier les distances annoncées dans l'itinéraire (Pontillas 50m?)
- [x] Sur la carte, faire mieux ressortir l'itinéraire calculé
- [x] Les flèches dans la bonne direction sur l'itinéraire
- [x] Ne pas masquer complètement les pistes hors itinéraire faut les dimmer (à 20% ?). Voir `map.rs` line 119
- [x] Problème du bouton bleu invisible sur Edge téléphone => le remonter. Voir `style/main.css` line 342 10rem
- [x] Créer un document ARCHITECTURE.md qui explique le "making of", les choix etc.
- [-] ISSUE CREATED - CI/CD faire en sorte de compiler en release et de pousser sur Heroku à chaque push sur GitHub
- [x] Recréer l'outil qui génère le `.json` dans le répertoire `data/`.
    * Doit être dans un sous workspace.
    * Le nom `serre_chevalier_YYYYMMDD_HHMMSS.json`.
    * Faut aller copier/coller/souder les codes qui sont dans
        * https://github.com/40tude/serre_che_proto/tree/main/get_data
        * https://github.com/40tude/serre_che_proto/tree/main/get_elevation
- [ ] Cliquer sur les noms de pistes ou de remontées
    * Nom, Lift ou Piste,
    * Preciser les infos du lift (platter, gondola 10p...)
    * ou le niveau de la piste (easy...),
    * Comme on clique sur une piste ou ue remonté peut on afficher les coordonnées et l'altitude (on fait une regression lineaire avec les 2 points autour?)
- [ ] Cliquer pour sélectionner le point de départ et le point d'arrivée (comment on gère si on clique au milieu de nulle part)


## Ideas

* Affichage 3D
* Planning semaine => stocker des données
* Plan pour visiter toute la semaine en N jours?
* ???
