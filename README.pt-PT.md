<div align="center">
  <img src="icono_circular.png" alt="Logótipo do LF Botonera de Efectos" width="96" height="96">
  <h1>LF Botonera de Efectos (Painel de efeitos sonoros)</h1>
  <p><strong>Os seus sons à mão, prontos para entrar no ar.</strong></p>
  <p>Um painel de efeitos sonoros para rádio, locução e emissões em direto. Organize separadores de estação, fundos musicais, chamadas e outros áudios de apoio numa aplicação para Windows e Linux, pensada para responder ao ritmo da emissão.</p>
</div>

[Español](README.md) · [English](README.en.md) · [Português (Brasil)](README.pt-BR.md) · **Português (Portugal)**

[Transferir](#transferir-e-instalar) · [Primeiros passos](#primeiros-passos) · [Funcionalidades](#funcionalidades) · [Ajuda](#ajuda-e-comunidade) · [Desenvolvimento](#compilar-o-código-fonte)

## O que é?

O **LF Botonera de Efectos** é uma aplicação para locutores, operadores de rádio, produtores, criadores de conteúdos e streamers que precisam de ter os seus sons disponíveis durante uma emissão. Distribua os áudios por uma grelha de botões e lance cada um no momento certo.

## Transferir e instalar

**[Obter na Microsoft Store](https://apps.microsoft.com/detail/9NJ8ST39QP7V)** (recomendado para Windows, com atualizações automáticas) · **[Transferências diretas para Windows e Linux](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/latest)**

| Sistema | Pacote | Quando escolher |
|---|---|---|
| Windows 10/11 | [Microsoft Store](https://apps.microsoft.com/detail/9NJ8ST39QP7V) | Instalação simples e atualizações geridas pela loja |
| Windows 10/11 | EXE | Instalação direta com assistente |
| Windows 10/11 | MSI | Ambientes empresariais ou com instalação gerida |
| Ubuntu / Debian | DEB | Distribuições compatíveis com pacotes Debian |
| Fedora / openSUSE | RPM | Distribuições compatíveis com pacotes RPM |
| Linux | AppImage | Executável portátil, sem instalação |

É necessário ter **Windows 10 20H2 ou posterior e WebView2 Runtime**, incluído no Windows 11 e nas versões atualizadas do Windows 10. Se utiliza a edição EXE/MSI, consulte o [guia de migração para a Microsoft Store (em espanhol)](Documentación/MIGRACION_MICROSOFT_STORE.md) para conservar as suas definições ao mudar de edição.

## Primeiros passos

1. Instale a aplicação e conclua o assistente de configuração inicial.
2. Arraste um ficheiro de áudio para um botão vazio e clique no botão para o reproduzir.
3. Clique com o botão direito do rato para editar o botão e atribuir um atalho de teclado.
4. Organize os seus programas ou sessões em separadores e perfis. Consulte o [manual do utilizador (em espanhol)](MANUAL.md) para mais informações.

## Funcionalidades

- **Os seus sons à mão:** botões personalizáveis, perfis, separadores, atalhos de teclado e controlo MIDI.
- **Música de fundo:** um reprodutor independente, com fila, volume e saída próprios. Parar os efeitos não interrompe a música.
- **Biblioteca e pesquisa:** encontre faixas e arraste-as para os botões ou para o reprodutor.
- **Preparação de áudio:** pré-escuta CUE, editor com forma de onda, ajuste de ganho e normalização.
- **Controlo Mic:** reduz temporariamente o nível dos destinos escolhidos, de forma manual ou automática. Não mistura, grava nem transmite o áudio do microfone.
- **Quatro idiomas na interface:** espanhol, inglês, português do Brasil e português de Portugal.

<details>
<summary><strong>Ver todas as funcionalidades</strong></summary>

### Organização e reprodução

- Perfis e separadores ilimitados, com grelhas de botões configuráveis; definições de áudio por perfil e dispositivo de saída por separador.
- Cores personalizadas nos botões e nos separadores, adaptadas aos temas claro e escuro; identificação com emojis e ícones.
- Motor de áudio nativo em Rust, com rodio e cpal, para reproduzir e misturar vários sons em simultâneo.
- Modos por botão: repetição em ciclo, sobreposição, reinício ao clicar e paragem dos restantes sons. Controlos globais Normal, Loop, Multi, Restart, Solo e Parar tudo.
- Reprodução sequencial de uma pasta: cada acionamento reproduz o ficheiro seguinte.
- Anúncios da hora compostos a partir dos seus ficheiros de áudio; locuções de temperatura e humidade com dados do Open-Meteo.

### Teclado, MIDI e pré-escuta

- Atalhos para botões, separadores e ações globais, incluindo os controlos do reprodutor. Os atalhos globais podem funcionar com a aplicação em segundo plano; o modo de mapeamento mostra as combinações na grelha.
- Seleção de entradas MIDI e atribuição de mensagens Note On, Control Change ou Program Change a botões, separadores e ações globais.
- Pré-escuta CUE noutro dispositivo, como auscultadores, para verificar uma faixa antes de a colocar no ar.
- Painel flutuante com o nome do áudio, progresso, tempo, volume, botão de paragem e navegação pela faixa com um clique.

### Editor de faixas

- Forma de onda que reflete o ganho aplicado e assinala a vermelho os trechos que sofreriam clipping; zoom de 1× a 30× através do controlo deslizante ou de Ctrl + roda do rato.
- Pontos de início e fim sem alterar o ficheiro original. Normalização opcional para −14 LUFS, com limite de pico de −1 dBFS, e ajuste manual de ganho em dB.
- Análise com indicação de progresso e cache persistente da forma de onda para voltar a abrir mais depressa as faixas já analisadas.
- Reprodução com reinício sem duplicar o áudio, paragem por etapas (pausar → voltar à marca → voltar a 0:00), reprodução a partir do cue e cursor animado a 60 fps.
- Editor integrado na janela principal ou aberto numa janela independente.
- Definições guardadas por ficheiro: o cue e o ganho aplicam-se a todos os botões que usam esse áudio e podem acompanhar os separadores e os perfis exportados.

### Biblioteca, metadados e cópias de segurança

- Janela de Biblioteca para percorrer Música, Efeitos, pastas indexadas e unidades locais. Pesquisa no painel fixo por nome do ficheiro, título, artista, álbum, género e palavras-chave.
- Atualização automática quando as pastas mudam. Reproduza no ar, ouça em CUE, adicione ao reprodutor ou arraste uma faixa para um botão.
- Edição individual ou em lote de metadados e etiquetas. Alteração opcional do nome dos ficheiros no disco e escrita opcional de metadados compatíveis numa faixa individual de Música.
- Remoção reversível de pastas do catálogo, sem apagar os áudios, com prazo de retenção de 30 a 365 dias.
- Cópia de segurança `.lfbackup` com as definições e a base de dados das faixas. **Os ficheiros de áudio não estão incluídos.** O restauro valida a cópia, guarda uma cópia de segurança do estado atual e reinicia a aplicação.

### Desempenho e interface

- Pré-carregamento de áudio na RAM para o perfil completo, o separador visível ou à medida que os áudios são utilizados. Cache de 32, 64, 128 ou 256 MB, com prazo configurável para remover faixas sem uso e acesso imediato ao cue dos áudios em cache.
- Temas claro, escuro e automático; arraste ficheiros de áudio, reorganize botões com Alt + arrastar e altere a ordem dos separadores por arrastamento.
- Medidor estéreo em tempo real, relógio, data, contagem decrescente, anular/refazer, tamanho do texto ajustável, assistente inicial e aviso de atualizações.
- Recuperação automática quando o dispositivo de áudio configurado não está disponível no arranque.

</details>

## Formatos de áudio compatíveis

MP3 · WAV · FLAC · OGG/Vorbis · OGG/Opus · AAC · M4A · AIFF. A descodificação utiliza o Symphonia, escrito em Rust, sem depender de componentes proprietários pagos.

## Compatibilidade com o LF Automatizador

Troque separadores de botões `.bdelf` e perfis `.bdeplf` com o [LF Automatizador](https://github.com/yosoyluisfernando/lf-automatizador). Cada aplicação ignora os campos exclusivos da outra. As exportações podem incluir cue e ganho como metadados opcionais (`bdelf_tracks`): a Botonera recupera-os na importação, adaptando os caminhos ao computador de destino; o LF Automatizador ignora-os.

## Compilar o código-fonte

São necessários o [Node.js 20+ com npm](https://nodejs.org/), o [Rust estável](https://rustup.rs/) e os [pré-requisitos do Tauri 2](https://tauri.app/start/prerequisites/) para o seu sistema.

Na pasta do repositório, execute `npm install` e depois `npm run tauri build`. Os instaladores ficam em `src-tauri/target/release/bundle/`. Para desenvolver com recarregamento automático, utilize `npm run tauri dev` ou `DEV.bat` no Windows.

Consulte [Compilação e versões (em espanhol)](Documentación/COMPILACION_Y_VERSIONES.md) para as instruções de compilação e o processo de publicação; a [Arquitetura (em espanhol)](Documentación/ARCHITECTURE.md) apresenta a organização do código. Leia o [guia de contribuição (em espanhol)](Documentación/CONTRIBUTING.md) antes de preparar uma alteração.

## Assinatura no Windows

Os instaladores diretos EXE/MSI não têm certificado de assinatura de programador, pelo que o Windows pode apresentar um aviso ao abri-los pela primeira vez. A Microsoft Store gere a assinatura e as atualizações da sua própria edição. O código-fonte é público e o projeto é software livre.

## Ajuda e comunidade

[Apoio e comunicação de problemas](SUPPORT.md) · [Sítio oficial](https://lfbotonera.blogspot.com/) · [Canal de novidades](https://t.me/+XKof2wDvGVw1YTRh) · [Grupo da comunidade](https://t.me/+bXppwWvJvSg5YjNh)

[Como contribuir](Documentación/CONTRIBUTING.md) · [Código de conduta](Documentación/CODE_OF_CONDUCT.md) · [Segurança](Documentación/SECURITY.md) · [Privacidade](PRIVACY.md) · [Avisos de terceiros](THIRD_PARTY_NOTICES.md)

A documentação dos links está principalmente em espanhol. O seletor de idioma altera o idioma desta apresentação.

## Apoiar o desenvolvimento

Se a aplicação lhe é útil nos seus programas, produções ou emissões, pode [apoiar a sua manutenção com um donativo através do PayPal](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ).

## Créditos de desenvolvimento

**Autor e criador: Luis Fernando Velásquez.**

Este projeto foi desenvolvido com o apoio de ferramentas de inteligência artificial:

- **[Claude AI](https://claude.ai)** (Anthropic): apoio na arquitetura Rust, implementação do motor de áudio, documentação técnica e revisão de código.
- **[Codex / GPT](https://openai.com)** (OpenAI): apoio nas fases iniciais do desenvolvimento.

As ferramentas de IA participaram como assistentes de desenvolvimento. O desenho do produto, as decisões de arquitetura, a direção criativa e a autoria do projeto pertencem ao autor.

## Licença

Copyright (C) 2026 **Luis Fernando Velásquez**. Software livre sob a licença **GNU GPL-3.0-or-later**, sem qualquer garantia. Pode redistribuí-lo e modificá-lo de acordo com os termos de [LICENSE](LICENSE).
