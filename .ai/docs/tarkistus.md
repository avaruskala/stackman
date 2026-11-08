# Tarkistus, Stackman

Tämä loki kirjaa aineisto-, koodi- ja arkkitehtuuritarkistuksissa havaitut ongelmat ja tehdyt ratkaisut.

## Aineisto- ja kooditarkistuksen löydöt

| # | Havainto | Luokka |
|---|----------|--------|
| 1 | Kansion `material/STACK_*` tiedostot alkavat suoraan `<question type="stack">`-elementillä `<quiz>`-juuren sijaan, ja `<name>` sisältää tekstin ilman `<text>`-alitägiä. | Aineistopoikkeama |
| 2 | Tiedosto `tentti-Test-top-20261107-1344.xml` sisältää Moodle 4+ -kenttiä `idnumber`, `questiondescription`, `feedbackstyle`, `description`, `Bool` ja `SetEquiv`, jotka puuttuivat alkuperäisestä `crates/stack/src/models/`-tietomallista. | Tietomallin puute |
| 3 | Tiedosto `johdatus_graafiteoriaan.xml` erottaa Maxima-sijoitukset pelkillä rivinvaihdoilla ilman `;`- tai `$`-lopetusmerkkiä. | Maxima-syntaksi |
| 4 | Sana `step` on Maximan varattu avainsana, minkä vuoksi `rand_with_step(lower, upper, step)` päätyy syntaksivirheeseen. | Maxima-syntaksi |
| 5 | CASTextin `{@solmu@}` palauttaa merkkijonon `"pienin"`, joka vääristyy, jos se kääritään `\(...\)`-matematiikkatilaan ja ajetaan `tex1`-funktiolla. | Renderöintisääntö |
| 6 | Tiedosto `crates/app/src/lib.rs` sisälsi yli 1 500 riviä, kun kaikki kysymystyypit ja näkymät olivat samassa tiedostossa. | Ylläpidettävyys |

## Käännös- ja rajatarkistuksen löydöt

- **Polkuturvallisuus säännössä 11.** Palvelinfunktio `load_sample_xml_file` tarvitsee tarkistuksen, joka estää `..`-polut ja rajaa luvun kansioon `material/` sekä tiedostoon `tentti-Test-top-20261107-1344.xml`.
- **Leptos 0.8 `<Suspense>` ja `Send`-vaatimus.** Kun `cargo test --workspace` kääntää koko työtilan, Cargo aktivoi yhtä aikaa ominaisuudet `ssr` ja `hydrate`. Syvästi sisäkkäiset näkymätyypit `<Suspense>`-komponentin sisällä ylittivät oletusrekursiorajan, ja tyyppi `Option<Children>` ei toteuttanut `Send`-ominaisuutta.
- **Viennin tarkistus säännössä 14.** Kysymyksen muokkaus ja XML-vienti tarvitsevat tarkistuksen, joka hylkää tyhjän kysymysnimen ja puuttuvat `[[input:X]]`-määrittelyt.

## Päätökset

| Havainto | Ratkaisu | Perustelu |
|----------|----------|-----------|
| Kolme XML-muotoa | Toteutettu sääntö 1 | Funktio `parse_moodle_xml` tukee `<quiz>`-, `<question>`- ja `SimpleStackQuestionXml`-rakennetta. |
| Moodle 4+ -kentät | Toteutettu sääntö 13 ja ADR 0002 | Uudet kentät lisättiin valinnaisina `Option`-kenttinä kansion `crates/stack/src/models/` tyyppeihin. |
| Puolipisteettömät Maxima-rivit | Toteutettu sääntö 5 | Funktio `normalize_maxima_statements` päättää lauseet rivinvaihdossa, kun sulkusyvyys on nolla. |
| Maximan `step`-avainsana | Korjattu | Funktio `rand_with_step(lower, upper, st)` käyttää parametrinimeä `st`. |
| Merkkijonot `{@...@}`-merkinnässä | Toteutettu sääntö 6 | Maxima tulostaa `stringp`-tarkistuksen tuloksen, ja merkkijonot sijoitetaan tekstiin ilman `\(...\)`-käärettä. |
| Suuri `crates/app/src/lib.rs` | Jaettu moduuleihin | Koodi jaettiin tiedostoihin `models.rs`, `api.rs`, `components/{header,sidebar,toolbar}.rs` ja `components/preview/{mod,stack,multichoice,truefalse}.rs`. |
| `<Suspense>` ja `Send`-raja | Korjattu | Komponentti `MoodleInfoBox` käyttää tyyppiä `Option<AnyView>` ja alikomponentit kutsuvat metodia `.into_any()` yhdessä asetuksen `#![recursion_limit = "256"]` kanssa. |
