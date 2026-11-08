# Testaus, Stackman

Tämä tiedosto määrittelee testiaineiston, deterministisen satunnaissiemenen, hyväksymiskriteerien ajotavan, kattavuusmatriisin ja testausvastuut.

## Testiaineisto

Kaikki testitasot käyttävät repossa olevia Moodle- ja STACK XML -tiedostoja sekä paikallista testi-Moodlea:

| Aineisto | Kuvaus |
|----------|--------|
| `tentti-Test-top-20261107-1344.xml` | Vuoden 2026 Moodle 4+ -vienti, jossa on 51 `<question>`-alkiota: 39 STACK-kysymystä, monivalintoja, tosi tai epätosi -kysymyksiä, kategoriaotsikoita, monikielisiä tekstejä ja Base64-kuvia. |
| `material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/*.xml` | Kahdeksan vuoden 2016 tiedostoa, jotka sisältävät `[[jsxgraph]]`-kuvaajia, matriisisyötteitä, `[[if test="..."]]`-ehtolohkoja ja puolipisteettömiä Maxima-rivejä. |
| `material/STACK_*/*.xml` | 28 yksinkertaistettua `<question type="stack">`-tiedostoa. |
| Kiinteä siemen `seed` | Oletusarvo `12345` käyttöliittymässä sekä `42` eräajotestissä `test_parse_and_eval_all_formats`. |
| Testi-Moodle | `http://10.134.240.190/`, jonka käyttäjätunnus ja salasana annetaan ympäristömuuttujina. |

## Deterministinen siemen ajossa ja testeissä

Crate `crates/stack` ei lue satunnaissiementä kellosta, kuten ADR 0001 määrää:

- Yksikkötesteissä jokainen kutsu antaa eksplisiittisen `seed: u32`-arvon, kuten `42` tai `12345`.
- Web-sovelluksessa oletussiemen on `12345`. Opettaja voi syöttää uuden luvun Siemen-kenttään tai valita arvon kysymyksen `<deployedseed>`-listasta.
- Painike `Uusi versio` laskee seuraavan siemenen kaavalla `seed.wrapping_mul(1664525).wrapping_add(1013904223) % 999999 + 1`, joten versiosarja on toistettavissa.

## Hyväksymiskriteerien ajo

Aja automaattiset testit ja käännöstarkistukset komennoilla:

```bash
cargo test --workspace
cargo leptos build
```

Tiedoston `crates/stack/src/render.rs` testi `test_parse_and_eval_all_formats` lukee kaikki 37 XML-tiedostoa ja tarkistaa, että jokainen STACK-kysymys evaluoituu `/usr/bin/maxima`-prosessissa arvolla `maxima_ok == true` eikä yksikään `[[input:...]]`-paikkamerkki jää korvaamatta.

## Kattavuusmatriisi

| Sääntö | Paikka | Spec ja kriteerit | Hylkäys- tai varatietesti |
|---|---|---|---|
| 1 Kolme XML-muotoa ja virheellinen XML | `stack::render::parse_moodle_xml` | 001, kriteerit 1, 2, 3, 4 | Kriteeri 4 testaa rikkinäisen XML-syötteen hylkäyksen |
| 2 `category` päivittää kategorian | `app::models::extract_questions_from_quiz` | 001, kriteerit 1, 5 | Ei hylkäystapausta |
| 3 Syötetyypit ja matriisin koko | `stack::render::evaluate_stack_question_maxima` | 002, kriteerit 2, 3 | Ei hylkäystapausta |
| 4 Deterministinen `seed` | `stack::render::build_maxima_script` | 002, kriteerit 1, 4 | Ei hylkäystapausta |
| 5 Puolipisteettömien Maxima-rivien normalisointi | `stack::render::normalize_maxima_statements` | 002, kriteeri 2 | Ei hylkäystapausta |
| 6 CASText `{@...@}` merkkijono ja LaTeX | `stack::render::render_castext` | 002, kriteerit 2, 5 | Ei hylkäystapausta |
| 7 Staattinen esikatselu kun Maxima epäonnistuu | `stack::render::render_stack_question_static` | 002, kriteeri 6 | Kriteeri 6 testaa varatien aktivoinnin |
| 8 Vastauspuun solmut ja lopetusarvo `-1` | `stack::models::stack::Node` | 002, kriteeri 7 | Ei hylkäystapausta |
| 9 Monivalinnan `single` ja `fraction` | `stack::models::multichoice` ja `app` | 002, kriteeri 8 | Ei hylkäystapausta |
| 10 Tosi tai epätosi `fraction` | `stack::models::truefalse` ja `app` | 002, kriteeri 9 | Ei hylkäystapausta |
| 11 Esimerkkipolun tarkistus | `app::api::load_sample_xml_file` | 001, kriteeri 7 | Kriteeri 7 testaa `..`-polun hylkäyksen |
| 12 Moodle-tunnukset eivät vuoda | `app::api` ja `crates/server` | 003, kriteerit 1, 2, 3 | Kriteeri 3 testaa virheellisen kirjautumisen |
| 13 Häviötön XML-vienti | `stack::models` | 004, kriteerit 1, 2 | Ei hylkäystapausta |
| 14 Muokatun kysymyksen tarkistus ennen vientiä | `stack::models` | 004, kriteerit 3, 4 | Kriteerit 3 ja 4 testaavat tyhjän nimen ja puuttuvan syötteen hylkäyksen |
| 15 STACK-JS- ja CASText-skriptilohkojen jäsennys | `stack::render::render_castext` ja `stack::render::collect_castext_expressions` | 005, kriteerit 1, 2, 3, 4, 5, 6 | Kriteeri 4 testaa irrallisen `[[script]]`- ja `[[style]]`-lohkon virheilmoituksen |

Jokaisella säännöllä on rivi matriisissa. Katselmoija tarkistaa muutokset tätä taulukkoa vasten.

## Milloin ja kuka

| Vaihe | Tekijä | Ajettava tarkistus | Tallennuspaikka |
|---|---|---|---|
| Askeleen aikana | Työntekijä | Oman craten testit `cargo test -p stack` tai `cargo check -p app` | Paluuviesti johtajalle |
| Ennen committia | Katselmoija | Diffin vertailu kattavuusmatriisiin, sääntöjen paikkoihin ja kieltoihin | Löytölista johtajalle |
| Ennen committia | Johtaja | `cargo leptos build` ja `cargo test --workspace` | Commit-viestin runko |
| Ennen pull requestin yhdistämistä | Ihminen | Specin kriteerit selaimessa ja testi-Moodlea `http://10.134.240.190/` vasten | Pull requestin runko |
| Ennen `master`-yhdistystä | Ihminen | Tiedoston `arkkitehtuuri.md` Valmis-osion tarkistus | Pull requestin runko |

## Mitä ei testata

- Maxima-prosessien suorituskykyä suuren käyttäjämäärän rinnakkaisessa kuormituksessa.
- Opiskelijan tenttisuorituksen aikarajoja, jotka kuuluvat Moodlen tenttiaktiviteetille.
