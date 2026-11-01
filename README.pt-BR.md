<div align="center">
  <img src="icono_circular.png" alt="Logo do LF Botonera de Efectos" width="96" height="96">
  <h1>LF Botonera de Efectos (Painel de efeitos sonoros)</h1>
  <p><strong>Seus sons a um clique, prontos para entrar no ar.</strong></p>
  <p>Um painel de efeitos sonoros para rádio, locução e transmissões ao vivo. Organize vinhetas, trilhas, chamadas e áudios de apoio em um aplicativo para Windows e Linux, feito para agilizar sua operação.</p>
</div>

[Español](README.md) · [English](README.en.md) · **Português (Brasil)** · [Português (Portugal)](README.pt-PT.md)

[Download](#download) · [Primeiros passos](#primeiros-passos) · [Recursos](#recursos) · [Ajuda](#ajuda-e-comunidade) · [Desenvolvimento](#compilar-o-código-fonte)

## O que é?

O **LF Botonera de Efectos** é um aplicativo para locutores, operadores de rádio, produtores, criadores de conteúdo e streamers que precisam ter seus áudios à mão durante uma transmissão. Distribua os sons em uma grade de botões e acione cada um na hora certa.

## Download

**[Baixar pela Microsoft Store](https://apps.microsoft.com/detail/9NJ8ST39QP7V)** (recomendado para Windows, com atualizações automáticas) · **[Downloads diretos para Windows e Linux](https://github.com/yosoyluisfernando/LF-Botonera-de-efectos/releases/latest)**

| Sistema | Pacote | Quando escolher |
|---|---|---|
| Windows 10/11 | [Microsoft Store](https://apps.microsoft.com/detail/9NJ8ST39QP7V) | Instalação simples e atualizações gerenciadas pela loja |
| Windows 10/11 | EXE | Instalação direta com assistente |
| Windows 10/11 | MSI | Ambientes corporativos ou com instalação gerenciada |
| Ubuntu / Debian | DEB | Distribuições compatíveis com pacotes Debian |
| Fedora / openSUSE | RPM | Distribuições compatíveis com pacotes RPM |
| Linux | AppImage | Executável portátil, sem instalação |

No Windows, é necessário ter **Windows 10 20H2 ou posterior e WebView2 Runtime**, incluído no Windows 11 e em versões atualizadas do Windows 10. Se você usa a edição EXE/MSI, consulte o [guia de migração para a Microsoft Store (em espanhol)](Documentación/MIGRACION_MICROSOFT_STORE.md) para manter suas configurações ao trocar de edição.

## Primeiros passos

1. Instale o aplicativo e conclua o assistente de configuração inicial.
2. Arraste um arquivo de áudio para um botão vazio e clique nele para reproduzir.
3. Clique com o botão direito para editar o botão e definir um atalho de teclado.
4. Organize seus programas ou sessões em abas e perfis. Consulte o [manual do usuário (em espanhol)](MANUAL.md) para saber mais.

## Recursos

- **Seus sons à mão:** botões personalizáveis, perfis, abas, atalhos de teclado e controle MIDI.
- **Música de fundo:** um player independente, com fila, volume e saída próprios. Parar os efeitos não interrompe a música.
- **Biblioteca e busca:** encontre faixas e arraste-as para os botões ou para o player.
- **Preparação de áudio:** pré-escuta CUE, editor com forma de onda, ajuste de ganho e normalização.
- **Controle Mic:** reduz temporariamente o nível dos destinos escolhidos, de forma manual ou automática. Não mistura, grava nem transmite o áudio do microfone.
- **Quatro idiomas na interface:** espanhol, inglês, português do Brasil e português de Portugal.

<details>
<summary><strong>Conheça todos os recursos</strong></summary>

### Organização e reprodução

- Perfis e abas ilimitados, com grades de botões configuráveis; ajustes de áudio por perfil e dispositivo de saída por aba.
- Cores personalizadas nos botões e nas abas, adaptadas aos temas claro e escuro; identificação com emojis e ícones.
- Motor de áudio nativo em Rust, com rodio e cpal, para reproduzir e misturar vários sons ao mesmo tempo.
- Modos por botão: repetição em loop, sobreposição, reinício ao clicar e interrupção dos demais sons. Controles globais Normal, Loop, Multi, Restart, Solo e Parar tudo.
- Reprodução sequencial de uma pasta: cada acionamento toca o próximo arquivo.
- Anúncios de hora montados com seus arquivos de áudio; locuções de temperatura e umidade com dados do Open-Meteo.

### Teclado, MIDI e pré-escuta

- Atalhos para botões, abas e ações globais, incluindo os controles do player. Os atalhos globais podem funcionar com o aplicativo em segundo plano; o modo de mapeamento exibe as combinações na grade.
- Seleção de entradas MIDI e atribuição de mensagens Note On, Control Change ou Program Change a botões, abas e ações globais.
- Pré-escuta CUE em outro dispositivo, como um fone de ouvido, para conferir a faixa antes de colocá-la no ar.
- Painel flutuante com nome do áudio, progresso, tempo, volume, botão de parada e navegação pela faixa com um clique.

### Editor de faixas

- Forma de onda que considera o ganho aplicado e destaca em vermelho os trechos que sofreriam clipping; zoom de 1× a 30× pelo controle deslizante ou Ctrl + roda do mouse.
- Pontos de início e fim sem alterar o arquivo original. Normalização opcional para −14 LUFS, com teto de pico de −1 dBFS, e ajuste manual de ganho em dB.
- Análise com indicador de progresso e cache persistente da forma de onda para reabrir mais rápido as faixas já analisadas.
- Reprodução com reinício sem duplicar o áudio, parada em etapas (pausar → voltar à marca → voltar a 0:00), reprodução a partir do cue e cursor animado a 60 fps.
- Editor integrado à janela principal ou aberto em uma janela independente.
- Ajustes salvos por arquivo: o cue e o ganho valem para todos os botões que usam aquele áudio e podem acompanhar as abas e os perfis exportados.

### Biblioteca, metadados e backup

- Janela de Biblioteca para navegar por Música, Efeitos, pastas indexadas e unidades locais. Busca no painel fixo por nome do arquivo, título, artista, álbum, gênero e palavras-chave.
- Atualização automática quando as pastas mudam. Reproduza no ar, ouça em CUE, adicione ao player ou arraste uma faixa para um botão.
- Edição individual ou em lote de metadados e tags. Renomeação opcional dos arquivos no disco e gravação opcional de metadados compatíveis em uma faixa individual de Música.
- Remoção reversível de pastas do catálogo, sem apagar os áudios, com prazo de retenção de 30 a 365 dias.
- Backup `.lfbackup` com as configurações e o banco de dados das faixas. **Os arquivos de áudio não estão incluídos.** A restauração valida o backup, cria uma cópia de segurança do estado atual e reinicia o aplicativo.

### Desempenho e interface

- Pré-carregamento de áudio na RAM para o perfil inteiro, a aba visível ou conforme o uso. Cache de 32, 64, 128 ou 256 MB, com prazo configurável para remover faixas sem uso e acesso imediato ao cue dos áudios em cache.
- Temas claro, escuro e automático; arraste arquivos de áudio, reorganize botões com Alt + arrastar e mude a ordem das abas por arraste.
- Medidor estéreo em tempo real, relógio, data, contagem regressiva, desfazer/refazer, tamanho de texto ajustável, assistente inicial e aviso de atualizações.
- Recuperação automática quando o dispositivo de áudio configurado não está disponível ao iniciar.

</details>

## Formatos de áudio compatíveis

MP3 · WAV · FLAC · OGG/Vorbis · OGG/Opus · AAC · M4A · AIFF. A decodificação usa o Symphonia, escrito em Rust, sem depender de componentes proprietários pagos.

## Compatibilidade com o LF Automatizador

Troque abas de botões `.bdelf` e perfis `.bdeplf` com o [LF Automatizador](https://github.com/yosoyluisfernando/lf-automatizador). Cada aplicativo ignora os campos exclusivos do outro. As exportações podem incluir cue e ganho como metadados opcionais (`bdelf_tracks`): a Botonera os recupera na importação, adaptando os caminhos ao computador de destino; o LF Automatizador os ignora.

## Compilar o código-fonte

Você precisa do [Node.js 20+ com npm](https://nodejs.org/), do [Rust estável](https://rustup.rs/) e dos [pré-requisitos do Tauri 2](https://tauri.app/start/prerequisites/) para seu sistema.

Na pasta do repositório, execute `npm install` e depois `npm run tauri build`. Os instaladores ficam em `src-tauri/target/release/bundle/`. Para desenvolver com recarga automática, use `npm run tauri dev` ou `DEV.bat` no Windows.

Consulte [Compilação e versões (em espanhol)](Documentación/COMPILACION_Y_VERSIONES.md) para as instruções de compilação e o processo de publicação; a [Arquitetura (em espanhol)](Documentación/ARCHITECTURE.md) apresenta a organização do código. Leia o [guia de contribuição (em espanhol)](Documentación/CONTRIBUTING.md) antes de preparar uma alteração.

## Assinatura no Windows

Os instaladores diretos EXE/MSI não têm certificado de assinatura de desenvolvedor, por isso o Windows pode exibir um aviso ao abri-los pela primeira vez. A Microsoft Store cuida da assinatura e das atualizações da sua própria edição. O código-fonte é público e o projeto é software livre.

## Ajuda e comunidade

[Suporte e relatos de problemas](SUPPORT.md) · [Site oficial](https://lfbotonera.blogspot.com/) · [Canal de notícias](https://t.me/+XKof2wDvGVw1YTRh) · [Grupo da comunidade](https://t.me/+bXppwWvJvSg5YjNh)

[Como contribuir](Documentación/CONTRIBUTING.md) · [Código de conduta](Documentación/CODE_OF_CONDUCT.md) · [Segurança](Documentación/SECURITY.md) · [Privacidade](PRIVACY.md) · [Avisos de terceiros](THIRD_PARTY_NOTICES.md)

A documentação dos links está principalmente em espanhol. O seletor de idioma muda o idioma desta apresentação.

## Apoie o desenvolvimento

Se o aplicativo ajuda nos seus programas, produções ou transmissões, você pode [apoiar sua manutenção com uma doação pelo PayPal](https://www.paypal.com/donate/?hosted_button_id=3JJVFFBVR4MQQ).

## Créditos de desenvolvimento

**Autor e criador: Luis Fernando Velásquez.**

Este projeto foi desenvolvido com o auxílio de ferramentas de inteligência artificial:

- **[Claude AI](https://claude.ai)** (Anthropic): auxílio na arquitetura Rust, implementação do motor de áudio, documentação técnica e revisão de código.
- **[Codex / GPT](https://openai.com)** (OpenAI): auxílio nas etapas iniciais do desenvolvimento.

As ferramentas de IA atuaram como assistentes de desenvolvimento. O design do produto, as decisões de arquitetura, a direção criativa e a autoria do projeto pertencem ao autor.

## Licença

Copyright (C) 2026 **Luis Fernando Velásquez**. Software livre sob a licença **GNU GPL-3.0-or-later**, sem qualquer garantia. Você pode redistribuí-lo e modificá-lo de acordo com os termos de [LICENSE](LICENSE).
