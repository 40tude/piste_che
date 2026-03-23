Peux tu m'aider à créer un prompt pour spec kit pour une nouvelle feature. Voici les idées importantes de la feature en question:
Recréer l'outil qui génère le `.json` dans le répertoire `data/`.
Ce projet doit être dans être dans un sous workspace nommé "data_generator" (si tu as un meilleur nom dis le moi)
Le nom du fichier devra être du style `serre_chevalier_YYYYMMDD_HHMMSS.json` et il devra être copié dans le dossier /data.
Le code est déjà écrit. Il faut aller copier/coller/fusionner les codes qui sont dans https://github.com/40tude/serre_che_proto/tree/main/get_data et
https://github.com/40tude/serre_che_proto/tree/main/get_elevation


Puis en court de discussion j'ai indiqué

1 resort_generator
2 le code existant est du rust
3 déclenchement manuel uniquement si besoin
4 get_data et get_elevation sont 2 CLI indépendants. L'idée ici c'est de simplifier et d'avoir un seul outil sous forme de CLI
Remarque : aujourd'hui on ne travaille QUE avec le domaine de Serre Chavalier. Faut être malin et anticiper le fait que plus tard, on utilisera l'outil resort_generator pour générer le
json de Chamonix ou de Montgenèvre.



