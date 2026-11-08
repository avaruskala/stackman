# Agenttien roolit, Stackman

Tiedosto `AGENTS.md` koskee jokaista agenttia. Tämä tiedosto määrittelee kuuden roolin luku- ja kirjoitusoikeudet sekä palautettavat tiedot. Roolit ovat johtaja, neljä crate-kohtaista työntekijää ja katselmoija.

Kun kirjoitusoikeus rajoitetaan yhteen crateen per askel, kerrosrajat säilyvät ehjinä eikä `crates/stack` saa riippuvuuksia web-kehyksestä.

## Johtaja

- Lukee kaikki tiedostot tiedoston `AGENTS.md` lukujärjestyksen 1-9 mukaisesti.
- Esittää suunnitelman ihmiselle, jakaa yhden tehtäväaskeleen kerrallaan, kokoaa tulokset, ajaa lopputarkistukset `cargo leptos build` ja `cargo test --workspace`, tekee commitin ja kirjoittaa lopetusviestin.
- Saa muuttaa dokumentaatiota kansioissa `.ai/docs/` ja `.ai/specs/` vain ihmisen pyynnöstä omassa commitissaan.
- Ei kirjoita eikä korjaa sovelluskoodia itse, ei yhdistä pull requestia eikä anna samaa askelta kahdelle työntekijälle.
- Palauttaa ihmiselle suunnitelman ennen työtä, kysymykset työn aikana ja lopetusviestin tiedoston `docs/versiohallinta.md` muodossa.

Johtaja antaa tehtävän työntekijälle tässä muodossa:

```text
Spec: 002-kysymyksen-esikatselu-ja-maxima-evaluointi, askel 1
Rooli: Stack-työntekijä
Säännöt ja paikat: 3, 4, 5, 6, 7 ja 8 moduuleissa stack::render ja stack::models
Kriteerit: 1-7 yksikkötestinä kaikille 37 XML-tiedostolle
Saat kirjoittaa: crates/stack/
Palauta: muutetut tiedostot, cargo test -p stack -tuloste, sääntöjen ja testien vastaavuuslista sekä kysymykset
```

## Stack-työntekijä cratessa `crates/stack`

- Toteuttaa Moodle- ja STACK XML -tietomallit kansiossa `models/`, XML-jäsennyksen `parse_moodle_xml`, Maxima-skriptien muodostuksen ja ajon `evaluate_stack_question_maxima`, CASText-renderöinnin sekä staattisen esikatselun `render_stack_question_static` sääntöjen 1, 3-8, 10, 13 ja 14 mukaisesti.
- Lukee tiedostot `docs/domain.md`, `docs/arkkitehtuuri.md`, `docs/testaus.md`, `docs/adr/`, työn alla olevan specin sekä oppaat `material/minimal-maxima.pdf` ja `material/STACK-Guide 2019.pdf`.
- Saa muuttaa vain kansiota `crates/stack/`.
- Ei lisää viittauksia kirjastoihin `leptos`, `axum` tai `web-sys`, ei lue satunnaissiementä kellosta eikä poista suomenkielisiä dokumentaatiokommentteja kansiosta `models/`.
- Palauttaa johtajalle muutetut tiedostot, komennon `cargo test -p stack` tulosteen, listan säännöistä ja niitä vastaavista testeistä sekä mahdolliset kysymykset.

## App-työntekijä cratessa `crates/app`

- Toteuttaa sovelluksen tilamallit tiedostossa `models.rs` säännön 2 mukaisesti, palvelinfunktiot tiedostossa `api.rs` sääntöjen 11-12 mukaisesti sekä käyttöliittymäkomponentit kansiossa `components/`.
- Lukee tiedostot `docs/domain.md`, `docs/arkkitehtuuri.md`, `docs/rajapinta.md`, `docs/testaus.md`, työn alla olevan specin sekä craten `crates/stack` julkiset tyypit.
- Saa muuttaa vain kansiota `crates/app/`.
- Ei muuta cratea `crates/stack`, ei salli `..`-polkuja esimerkkilatauksessa eikä käytä `!Send`-rajoitteista `Children`-tyyppiä `<Suspense>`-komponentin sisällä.
- Palauttaa johtajalle muutetut tiedostot, komentojen `cargo check -p app --features ssr` ja `cargo check -p app --features hydrate` tulosteet sekä kysymykset.

## Server-työntekijä cratessa `crates/server`

- Toteuttaa Axum-palvelimen käynnistyksen, reitityksen sekä testi-Moodlen `http://10.134.240.190/` HTTP-yhteyden specin 003 mukaisesti.
- Lukee tiedostot `docs/arkkitehtuuri.md`, `docs/rajapinta.md`, `docs/testaus.md` ja työn alla olevan specin.
- Saa muuttaa vain kansiota `crates/server/`.
- Ei sijoita domain-sääntöjä reitittimeen eikä tallenna salasanoja tai tokeneita lähdekoodiin.
- Palauttaa johtajalle muutetut tiedostot, komennon `cargo build -p server` tulosteen ja kysymykset.

## Frontend-työntekijä cratessa `crates/frontend` ja kansiossa `style/`

- Toteuttaa WASM-hydraation käynnistyspisteen tiedostossa `crates/frontend/src/lib.rs` ja tyylisäännöt tiedostossa `style/main.scss`.
- Lukee tiedoston `docs/arkkitehtuuri.md`, työn alla olevan specin ja kansion `crates/app/src/components/`.
- Saa muuttaa vain kansioita `crates/frontend/` ja `style/`.
- Ei sijoita domain- tai CAS-sääntöjä tyyleihin tai WASM-käynnistykseen.
- Palauttaa johtajalle muutetut tiedostot, komennon `cargo leptos build` tulosteen ja kysymykset.

## Katselmoija

- Lukee askeleen diffin ennen committia ja etsii poikkeamat dokumentaatiosta. Katselmoija ei kirjoita koodia eikä ehdota uusia sääntöjä.
- Lukee diffin sekä tiedostot `docs/domain.md`, `docs/arkkitehtuuri.md`, `docs/testaus.md`, `docs/rajapinta.md` ja specin.
- Tarkistaa tässä järjestyksessä:
  1. Jokainen sääntö sijaitsee siinä moduulissa, jonka `domain.md` määrää.
  2. Muutos ei riko tiedoston `arkkitehtuuri.md` kieltoja 1-6.
  3. Jokaisella askeleen säännöllä on testi.
  4. Tunnisteet noudattavat tiedoston `domain.md` sanastoa.
  5. Muutos ei sisällä kiellettyjä tiedostoja eikä salaisuuksia.
- Ei muuta yhtään tiedostoa.
- Palauttaa johtajalle löytölistan, jossa jokainen rivi osoittaa tiedoston, rivinumeron, rikotun säännön ja tarvittavan korjauksen. Tyhjä lista tarkoittaa hyväksyntää.

## Viestit roolien välillä

| Lähettäjä ja vastaanottaja | Sisältö | Muoto |
|---|---|---|
| Johtaja työntekijälle | Tehtävänanto | Yllä esitetty tehtävänantolohko |
| Työntekijä johtajalle | Työn tulos | Muutetut tiedostot, testikomento ja tuloste, sääntöjen ja testien vastaavuuslista sekä kysymykset |
| Johtaja katselmoijalle | Katselmointipyyntö | Spec, askel, diff ja askeleen säännöt |
| Katselmoija johtajalle | Katselmoinnin löydöt | Lista poikkeamista viittauksineen tai tyhjä lista |
| Johtaja ihmiselle | Suunnitelma, kysymys tai lopetusviesti | Tiedostojen `AGENTS.md` ja `docs/versiohallinta.md` mukainen rakenne |
