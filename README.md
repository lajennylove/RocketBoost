# Rocket Boost home page

The home page and documentation index for [rocketboost.rs](https://rocketboost.rs), the RocketBooster family of projects for Rocket.rs. It is a Rocket app that renders [Propel](https://github.com/lajennylove/propel) views, with Tailwind and Vite for the styles, built the same way as the Propel and Ignite docs sites.

Pages: `/` (home), `/about` (what the family is, and what is not built yet), `/docs` (links to the Propel and Ignite docs). Any other path is a 404 page.

The docs live on [propel.rocketboost.rs](https://propel.rocketboost.rs) and [ignite.rocketboost.rs](https://ignite.rocketboost.rs). This site links to them and does not repeat them.

## Dependencies

- Rocket: git revision `3a54d079` (the commit is `3a54d079aef060a8f732bd04ea54b0581a604087`, pinned in `Cargo.lock`)
- Propel: git revision `e0fb10893095a4b66996d35efb45cf0dd466ccdd` (`propel_rocket`, `propel_runtime`, `propel_compiler`)

## Run

```sh
npm install
npm run build          # once: Rocket serves /build from public/build
npm run dev            # Vite, in one terminal (debug builds load the styles from it)
cargo run              # Rocket, in another; a debug build reads the views from resources/views
cargo build --release  # a release build compiles the views in build.rs
```

The views are compiled into the release binary, so a view edit needs a rebuild there. In a debug build they are read on each request.

## Analytics

Every page has the GoatCounter script just before `</body>`, in `resources/views/layouts/app.propel`.

## Deploying

Not set up. The root site on the server is still a static HTML page. Serving this Rocket app from rocketboost.rs needs a decision, and no deploy step has been run.
