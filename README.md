# Gourmestre

## Stack

* Rust (Hyper, Tokio, http-body-util)
* PostgreSQL
* HTMX, Tailwind
* Docker

## Roadmap

-[x] create custom routes  
-[x] Use custom handle error  
-[x] use .env  
-[x] add the PostregSQL db  
-[x] authentication management  
-[x] session handler  
-[x] dashboard for conected users  
-[ ] review data structure  
-[ ] unit and accaptance test  
-[ ] add a "bin.rs" to launch migrations or admin things  
-[ ] refacto with separated routes  
-[ ] Use HTTP/2  
-[ ] Une expiration automatique de session  
-[ ] La suppression côté serveur lors du logout  
-[ ] Un middleware rust pour protéger plusieurs routes sans dupliquer la logique

## Amélioration possible

* Formulaire "Ajout rapide" modernisé, inputs plus accessibles et responsive.
* Accessibilité : navigation par bouton, labels, aria-labels.
* htmx dans les layers pour charger dynamiquement le contenu sans JS custom.

## PostgreSQL

* se connecter au conteneur : `docker exec -it <id du conteneur> /bin/bash`
* se connecter à la BD : `psql -U u_gourmestre -d Gourmestre`
* lister les tables : `\dt`
* regarder le détail de la table user : `\d users`
* lister les utilisateurs enregistrer : `SELECT * FROM users;`
* actualiser la BD : `psql -U u_gourmestre -d Gourmestre -f docker-entrypoint-initdb.d/init.sql`
