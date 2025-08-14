## FlowVersion TypeScript CLI（実験版・日本語ドキュメント）

Rust 実装に先立ち、Windows 環境で手軽に試せる TypeScript 版の簡易 CLI を提供します。変更の意図（intention）や目的（goal）などのメタ情報を付けて、ファイルスナップショットをコミットできます。

- コマンド名: `flow` または `flow-ts`
- 主な機能: init / config / add / status / diff / unstage / reset / commit / log / show / checkout
- データ保存: `.flow/` 配下（JSON/YAML + オブジェクトストア）

---

## クイックスタート

PowerShell（pwsh）を想定しています。

```powershell
cd C:\dev\FlowVersion\ts-cli
npm install
npm run build

# リポジトリ初期化
npx flow-ts init --name "my-project"

# 追加とコミット
npx flow-ts add src/**/*.ts -i "初期コード追加"
npx flow-ts commit -i "初期実装" -c 0.9 --goal "最小MVP" --context "PoC" --impact "Low"

# ログと表示
npx flow-ts log
npx flow-ts show <commitIdPrefix>
```

グローバルに使う場合は link して `flow` コマンドでも使えます。

```powershell
npm run build
npm link
flow --help
```

---

## コマンド一覧と使い方

### init
- 説明: `.flow/` を作成し、メタ情報と設定ファイルを初期化します。
- 例:
	- `flow init --name "my-project" -d "説明文"`

### config
- `config show` 現在の設定（config.yml）をJSONで表示
- `config get <key>` キーの値を表示（存在しない場合は空）
- `config set <key> <value>` 値を保存

### add
- 説明: ファイルをステージ（インデックス）に追加。glob対応。
- 主なオプション:
	- `-i, --intention <text>` 変更の意図を付与
- 例:
	- `flow add README.md -i "ドキュメント更新"`
	- `flow add src/**/*.ts`

### status
- 説明: ステージ状況を表示。作業ツリーとの差分があれば `modified`、存在しない場合は `deleted` を表示。

### diff <file>
- 説明: 作業ツリーとステージ済みバージョンの差分（Unified Diff）を表示。

### unstage <file>
- 説明: 指定ファイルをステージから外します。

### reset
- 説明: ステージをクリアします。

### commit
- 説明: ステージされたファイルのスナップショットをコミットに保存。
- 主なオプション:
	- `-i, --intention <text>` 変更の意図
	- `-c, --confidence <num>` 0..1 の信頼度（既定: 1）
	- `--goal <text>` 変更の目標
	- `--context <text>` 背景/文脈
	- `--impact <text>` 期待される影響

### log
- 説明: コミット履歴表示。IDは短縮、confidence は色分け（高: 緑、中: 黄、低: 赤）。

### show <idOrPrefix> [file]
- 説明: コミットのメタ情報とスナップショットを表示。`file` を指定するとその内容を出力。
- メモ: 短いIDで曖昧な場合は候補が提示されます。

### checkout <idOrPrefix>
- 説明: スナップショットから作業ツリーに復元。
- オプション:
	- `-p, --path <file>` 単一ファイルを復元
	- `-A, --all` スナップショット全体を復元
	- `-f, --force` 既存ファイルを上書き
	- `-n, --dry-run` 書き込みは行わず計画のみ表示
- 例:
	- `flow checkout abc12345 -p src/index.ts`
	- `flow checkout abc12345 -n -A`（ドライラン）

---

## `.flow/` データ構造

- `objects/`: コンテンツアドレス化されたブロブ（SHA-256）を保存
- `index.json`: ステージ（パス、意図、sha、サイズ、mtime 等）
- `commits.json`: コミット配列（id、intention、confidence、timestamp、goal/context/impact、snapshot）
- `config.yml`: リポジトリ設定（名前や説明など）

JSON の読み書きは、BOM 除去・一時ファイル書き込み・バックアップ作成を行う安全なI/Oで行っています（破損時の復旧を優先）。

---

## よくある注意点

- コミットIDは接頭辞で指定できますが、曖昧な場合は候補が表示されます。より長いIDを指定してください。
- `checkout` は既存ファイルがあると上書きしません（`--force` 指定で上書き）。変更内容だけを確認したい場合は `--dry-run` を使ってください。
- `add` は glob を解釈します。意図せず大量に追加しないようパターンに注意してください。

---

## トラブルシューティング

- 「No commits yet.」: まだコミットがありません。`add` でステージし、`commit` を実行してください。
- 「Not staged」: `diff` 対象がステージにありません。`add` してください。
- JSON の破損が疑われる場合: 自動バックアップ（`.bak` 等）から復旧が試みられます。うまくいかない場合は `.flow/` をバックアップの上、手動修復をご検討ください。

---

## ライセンス / ステータス

このCLIは実験的です。仕様は予告なく変更される可能性があります。バグ報告・提案歓迎です。
