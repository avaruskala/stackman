# Spec 003, Moodle-instanssin integraatio

## Tarina

Opettajana haluan tuoda kysymyskategorioita ja kysymyksiä suoraan testi-Moodlesta `http://10.134.240.190/` Stackmanin esikatseluun sekä viedä tarkastetun kysymyspankin takaisin Moodleen ilman manuaalista tiedostojen lataamista ja siirtämistä selaimessa.

## Käsitteet

Sanasto ja kentät on määritelty tiedostossa `docs/domain.md`.

## Tekemiset

| Toiminto | Onnistumisen ehto | Epäonnistumisen seuraus |
|----------|-------------------|-------------------------|
| Yhdistä testi-Moodleen ja listaa kysymyskategoriat | Palvelin muodostaa yhteyden Moodle-instanssiin ympäristömuuttujilla ja palauttaa kurssin sekä kategorioiden tiedot. | Jos Moodle ei vastaa tai kirjautuminen epäonnistuu, palvelinfunktio palauttaa virheen `ServerFnError` ilman salasanaa tai tokenia. |
| Tuo kysymykset Moodlesta esikatseluun | Palvelin hakee valitun kategorian Moodle XML -viennin, jäsentää sen funktiolla `parse_moodle_xml` ja palauttaa monikon `(lähdenimi, Quiz)` kysymyslistaan. | Jos XML-vienti on tyhjä tai virheellinen, käyttöliittymä näyttää virheen `StatusBanners`-komponentissa säännön 1 mukaisesti. |
| Vie kysymyspankki Moodleen | `Quiz` läpäisee säännön 14 tarkistuksen ja palvelin lähettää XML-aineiston Moodlen kysymyspankkiin. | Jos tarkistus tai Moodlen tuontirajapinta hylkää pyynnön, käyttöliittymä näyttää virhesyyn. |

## Säännöt ja paikat

Viittaukset tiedoston `docs/domain.md` sääntöihin:

- Sääntö 1, XML-vastauksen jäsennys `Quiz`-rakenteeksi, sijaitsee funktiossa `stack::render::parse_moodle_xml`.
- Sääntö 2, `category`-alkioiden muunnos kategorioiksi, sijaitsee funktiossa `app::models::extract_questions_from_quiz`.
- Sääntö 12, Moodle-salasanoja tai tokeneita ei tallenneta repoon eikä lähetetä selaimelle, sijaitsee moduulissa `app::api` ja cratessa `crates/server`.
- Sääntö 13, kaikki Moodle 4+ -kentät säilyvät viennissä, sijaitsee moduulissa `stack::models` ADR 0002:n mukaisesti.

## Hyväksymiskriteerit

Testi-Moodle on käynnissä osoitteessa `http://10.134.240.190/` tai yksikkötestissä käytetään paikallista testipalvelinta.

1. **Alkutila:** Palvelimella on asetettu testi-Moodlen yhteysasetukset `MOODLE_URL=http://10.134.240.190/`. **Toiminto:** Opettaja tuo kysymyspankin Moodlesta. **Lopputulos:** Kysymykset ilmestyvät vasemman reunan listaan ja ovat esikatseltavissa samalla tavalla kuin paikallisesta XML-tiedostosta ladatut kysymykset.
2. **Alkutila:** Käyttäjä tekee minkä tahansa Moodlen tuonti- tai vientikutsun. **Toiminto:** Tarkastetaan selaimelle palautettava HTTP-vastaus ja lähdekoodi. **Lopputulos:** Vastauksessa, URL-parametreissa tai repossa ei esiinny Moodlen salasanaa eikä tokenia säännön 12 mukaisesti.
3. **Alkutila:** Palvelimella on virheellinen Moodle-salasana tai Moodle-palvelin ei vastaa. **Toiminto:** Opettaja yrittää hakea kysymyksiä Moodlesta. **Lopputulos:** Palvelinfunktio palauttaa virheen, jonka käyttöliittymä näyttää virhepalkissa kaatumatta.
4. **Alkutila:** Stackmanissa on kelvollinen `Quiz`. **Toiminto:** Opettaja vie sen testi-Moodleen ja hakee saman kategorian takaisin. **Lopputulos:** Kysymysten lukumäärä, tyypit, `questionvariables` ja vastauspuut täsmäävät alkuperäiseen.

## Ei-tavoitteet

- Sovellus ei hae opiskelijoiden tenttisuorituksia eikä arvosanoja Moodlesta.
- Sovellus ei hallinnoi Moodlen käyttäjiä eikä luo uusia kursseja.

## Tehtävät

1. **Server-työntekijä cratessa `crates/server`.** Toteuta Moodle-asiakasmoduuli säännön 12 mukaisesti. Lue yhteysasetukset ympäristömuuttujista `MOODLE_URL`, `MOODLE_USERNAME` ja `MOODLE_PASSWORD` tai `MOODLE_TOKEN`. Askel on valmis, kun integraatiotesti menee läpi eikä koodissa ole kovakoodattuja tunnuksia.
2. **App-työntekijä tiedostossa `crates/app/src/api.rs`.** Toteuta `#[server]`-funktiot `fetch_moodle_questions` ja `import_questions_to_moodle`, jotka jäsentävät saadun XML-aineiston funktiolla `parse_moodle_xml`. Askel on valmis, kun funktiot kääntyvät sekä `ssr`- että `hydrate`-ominaisuudella.
3. **App-työntekijä tiedostossa `crates/app/src/components/header.rs`.** Lisää Moodle-tuonnin ja -viennin painikkeet sekä tilailmoitukset. Askel on valmis, kun opettaja voi ladata kysymykset testi-Moodlesta käyttöliittymän kautta.
