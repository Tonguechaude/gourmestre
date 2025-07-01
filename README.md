# Gourmestre

## Stack

* Rust (Axum, Askama, SQLx)
* PostgreSQL
* HTMX

## Roadmap

* create custom routes
* Use HTTP/2
* Use custom handle error
* use .env
* add the PostregSQL db

## PostgreSQL

* se connecter au conteneur : `docker exec -it <id du conteneur> /bin/bash`
* se connecter à la BD : `psql -U u_gourmestre -d Gourmestre`
* lister les tables : `\dt`
* regarder le détail de la table user : `\d users`
* lister les utilisateurs enregistrer : `SELECT * FROM users;`
