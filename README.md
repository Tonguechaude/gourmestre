# Gourmestre

## Stack

* Rust (Axum, Askama, SQLx)
* PostgreSQL
* HTMX

## Roadmap

-[x] create custom routes
-[x] Use custom handle error
-[x] use .env
-[x] add the PostregSQL db
-[ ] authentication management
-[ ] session handler
-[ ] dashboard for conected users
-[ ] review data structure
-[ ] unit and accaptance test
-[ ] add a "bin.rs" to launch migrations or admin things
-[ ] refacto with separated routes
-[ ] Use HTTP/2

## PostgreSQL

* se connecter au conteneur : `docker exec -it <id du conteneur> /bin/bash`
* se connecter à la BD : `psql -U u_gourmestre -d Gourmestre`
* lister les tables : `\dt`
* regarder le détail de la table user : `\d users`
* lister les utilisateurs enregistrer : `SELECT * FROM users;`
