# Arkkitehtuuri, Stackman

## Osat

Työtilassa on neljä Rust-cratea kansiossa `crates/`.

| Crate | Kerros | Vastuut |
|-------|--------|---------|
| `crates/stack` | Domain ja jäsentäjä | Moodle- ja STACK XML -mallit kansiossa `models/`, XML-jäsennys `parse_moodle_xml`, CASText-renderöinti, Maxima-skriptin muodostus ja suoritus `evaluate_stack_question_maxima` sekä staattinen esikatselu `render_stack_question_static`. Vastaa säännöistä 1, 3-8, 10, 13 ja 14. |
| `crates/app` | Sovellus ja käyttöliittymä | Leptos-sovelluslogiikka ja käyttöliittymämoduulit: `models` vastaa suodattimista, `QuestionEntry`-rakenteesta ja säännöstä 2, `api` vastaa palvelinfunktioista ja säännöistä 11-12 sekä `components` vastaa käyttöliittymän osista. |
| `crates/server` | Palvelin | Axum-webpalvelin, staattisten tiedostojen jakelu kansiosta `public/`, Leptos SSR -reititys ja ympäristömuuttujien luku. |
| `crates/frontend` | Selainpää | WebAssembly-käynnistyspiste, joka kytkee selaimen tapahtumankäsittelijät, tiedostonluvun, MathJaxin ja JSXGraphin DOM-puuhun. |

### Craten `crates/app` sisäinen moduulijako

Kansion `crates/app/src/` tiedostot on jaettu vastuualueen mukaan:

- `lib.rs` sisältää HTML-rungon `shell`, reitittimen `App` ja tilakoordinaattorin `HomePage`.
- `models.rs` sisältää tyypit `QuestionEntry`, `QuestionTypeFilter` ja `LanguageFilter` sekä funktion `extract_questions_from_quiz`.
- `api.rs` sisältää Leptos `#[server]`-funktiot sekä selaimen tiedosto- ja renderöintikutsut.
- `components/header.rs` sisältää yläpalkin `AppHeader` ja ilmoituspalkin `StatusBanners`.
- `components/sidebar.rs` sisältää suodattimet ja kysymyslistan `QuestionSidebar` sekä rivikomponentin `QuestionListItem`.
- `components/toolbar.rs` sisältää esikatselun työkalupalkin `PreviewToolbar`.
- `components/preview/` sisältää kysymystyyppikohtaiset näkymät `stack.rs`, `multichoice.rs` ja `truefalse.rs` sekä jaetun tietolaatikon `MoodleInfoBox` tiedostossa `mod.rs`.

## Riippuvuuksien suunta

Riippuvuudet osoittavat sisäänpäin kohti cratea `crates/stack`:

- `crates/stack` ei riipu kirjastoista `leptos`, `axum`, `tokio` tai `web-sys`.
- `crates/app` riippuu cratesta `crates/stack`. Palvelinriippuvuus `tokio` kääntyy vain ominaisuudella `ssr` ja selainriippuvuudet `web-sys`, `js-sys` ja `wasm-bindgen-futures` kääntyvät vain ominaisuudella `hydrate`.
- `crates/server` riippuu cratesta `crates/app` ominaisuudella `ssr`, eikä se viittaa crateen `crates/frontend`.
- `crates/frontend` riippuu cratesta `crates/app` ominaisuudella `hydrate`, eikä se viittaa crateen `crates/server`.

Kääntäjä ja Cargo-ominaisuudet valvovat näitä rajoja.

## Kiellot

1. Älä lisää web-kehystä tai selainrajapintoja crateen `crates/stack`.
2. Älä lue satunnaissiementä kellosta tai globaalista tilasta cratessa `crates/stack`. Funktio ottaa aina parametrin `seed: u32`, kuten ADR 0001 määrää.
3. Älä anna epäonnistuneen `/usr/bin/maxima`-suorituksen kaataa esikatselua, vaan palauta staattinen näkymä `render_stack_question_static` säännön 7 mukaisesti.
4. Älä hyväksy mielivaltaista tiedostopolkua funktiossa `load_sample_xml_file` säännön 11 vastaisesti.
5. Älä kovakoodaa testi-Moodlen `http://10.134.240.190/` salasanoja tai tokeneita lähdekoodiin tai WASM-pakettiin säännön 12 vastaisesti.
6. Älä sijoita usean kysymystyypin näkymiä samaan tiedostoon cratessa `crates/app`.

## Elinkaari

Ladatut kysymykset `Vec<QuestionEntry>` ja suodatinvalinnat säilyvät Leptos-signaaleissa selaimen istunnon ajan. STACK-kysymyksen CAS-evaluointi käynnistyy `Resource`-hakuna aina, kun valittu STACK-kysymys tai `seed`-signaali muuttuu. Palvelin ajaa jokaisen Maxima-evaluoinnin omassa `/usr/bin/maxima --very-quiet` -aliprosessissaan `tokio::task::spawn_blocking`-kutsun sisällä, ja prosessi päättyy heti tuloksen valmistuttua.

## Ei-tavoitteet

- Sovellus ei tallenna opiskelijoiden tenttisuorituksia eikä arvosanoja.
- Sovellus ei tallenna XML-tiedostoja SQL-tietokantaan.
- Sovellus ei tarjoa vapaata Maxima-komentoriviä selaimessa, vaan suorittaa vain kysymyksen omat muuttujat, CASText-lausekkeet ja mallivastaukset.

## Valmis

Järjestelmä täyttää valmiuden ehdot, kun seuraavat väitteet pitävät paikkansa:

- Kaikki 37 XML-tiedostoa kansioissa `material/` ja `tentti-Test-top-20261107-1344.xml` jäsentyvät ja evaluoituvat virheittä.
- Sama `seed`-arvo tuottaa jokaisella ajokerralla samat muuttujien arvot, samat mallivastaukset ja saman kysymystekstin.
- Jos Maxima ei ole käytettävissä, esikatselu näyttää staattisen näkymän ja tilamerkin `Staattinen esikatselu` kaatumatta.
- Funktio `load_sample_xml_file` hylkää polunylitysyrityksen `../Cargo.toml`.
- Craten `crates/stack` yksikkötestit `cargo test -p stack` menevät läpi ilman käynnissä olevaa web-palvelinta tai selainta.
