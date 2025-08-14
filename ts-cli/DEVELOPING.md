# 開発者向けガイド（FlowVersion TypeScript CLI）

このドキュメントは、`ts-cli` パッケージの開発者が素早く着手できるように、ビルド・実行・設計方針・コード規約・拡張方法・品質チェックの要点をまとめたものです。

---

## 開発環境

- Node.js: 22 以上（本リポジトリでは v22 系で動作確認）
- パッケージマネージャ: npm
- OS: クロスプラットフォーム（Windows 前提で検証）。パス区切りは内部的に POSIX 化しています。

初回セットアップ

```powershell
cd C:\dev\FlowVersion\ts-cli
npm install
```

---

## ビルド & 実行

- 開発時（ソース直実行）
  ```powershell
  npm run dev          # tsx で src/index.ts を直接実行
  npx tsx src/index.ts --help
  ```
- ビルド（配布物生成）
  ```powershell
  npm run build        # tsup により ESM/CJS + dts を生成
  node dist/index.cjs --help
  ```
- グローバルリンク（ローカル CLI として）
  ```powershell
  npm run build
  npm link
  flow --help          # または flow-ts
  ```

---

## プロジェクト構成（抜粋）

```
src/
  index.ts              # CLIエントリ（commander定義：init/config/add/...）
  utils/
    objectStore.ts      # CAS（SHA-256）でのオブジェクト保存/読み出し
    paths.ts            # .flow 配下のパス解決（flowDir/objectsDir/index/commits）
    config.ts           # config.yml の読み書き
    json.ts             # JSON の安全な読み書き（BOM除去/バックアップ/atomic write）
    hash.ts             # 予備のハッシュ関数（必要に応じて利用）
  types/*.d.ts          # 外部ライブラリの型補助
```

- ビルド: `tsup`（ES2022 目標、esm/cjs 出力、dts 生成）
- CLI: `commander`
- 表示: `picocolors`
- 依存: `diff`（Unified Diff 出力）, `fast-glob`（add のパターン展開）, `yaml`（config.yml）

---

## データモデルと設計

- .flow/ 配下
  - `objects/`: コンテンツアドレスストア（sha256 を2桁/残り で分割保存）
  - `index.json`: ステージ（IndexEntry[]）
  - `commits.json`: コミットの配列（Commit[]）
  - `config.yml`: リポジトリ設定

- IndexEntry
  - path: 正規化パス（/ 区切り）
  - intention: 変更の意図（null 可）
  - sha: ステージされた内容のオブジェクトID（sha256）
  - size, mtime, addedAt: 参照用のメタ

- Commit
  - id, intention, confidence, timestamp
  - goal/context/impact（任意）
  - snapshot: IndexEntry[]（コミット時点のステージ内容）

- 方針
  - JSON I/O は `readJsonSafe`/`writeJsonSafe` 経由で行い、破損に強く（BOM 除去 / バックアップ / 一時ファイルからの置換）
  - パスは内部表現を POSIX スラッシュで統一（作業ツリー I/O 時のみOSに合わせて変換）
  - 変更の流れ：作業ツリー → add でステージ → commit で snapshot 固定 → show/log/checkout で参照

---

## コーディング規約（軽量）

- TypeScript: `strict` 前提のつもりで書く（実際の tsconfig は ES2022/バンドラ設定）。
- 例外処理: ファイル I/O は try/catch を基本に、ユーザに分かる短文でフィードバック。
- JSON I/O: 必ず utils/json.ts を使う。
- パス操作: 入力は `normalizeRel` → 内部 `/` 固定、保存/比較はこの正規化を前提。
- CLI 出力: `picocolors` で最低限の色付け。冗長にしない。
- 依存追加: 小さく保ち、ロックファイルを更新。

---

## 機能追加ガイド（例: 新しいサブコマンド）

1) `src/index.ts` にコマンド定義を追加（`program.command('xxx')...`）。
2) 入出力・エラーケースの軽い契約を考える：
   - 入力: 引数/オプション必須性、既定値
   - 出力: 成功メッセージ、結果フォーマット
   - エラー: ファイル未存在、フォーマット不正、競合（曖昧ID 等）
3) 既存のユーティリティ（`paths.ts`, `json.ts`, `objectStore.ts`）を活用。
4) データ永続化が必要なら JSON/YAML のどちらかを採用し、既存ファイルの互換性に注意。
5) パフォーマンス・大規模入力を考慮（glob の件数、バイナリサイズ、差分表示の妥当性）。

---

## 代表的なエッジケース

- 空のステージ: `commit` は拒否（案内を表示）。
- 曖昧なコミットID: 候補提示。より長いIDを求める。
- 既存ファイル衝突: `checkout` は既定スキップ、`--force` で上書き。
- JSON 破損: `readJsonSafe` が復旧を試みる。失敗時は空配列/空オブジェクトで続行しつつ警告（実装に応じて）。
- バイナリ/巨大ファイル: `diff` 表示のコストや可読性に留意。

---

## 品質ゲート（Build / Lint / Smoke）

- Build: `npm run build`（失敗をゼロに）
- Lint: `npm run lint`（ESLint 設定がある場合）
- スモークテスト（例）
  ```powershell
  # 初期化
  npx tsx src/index.ts init --name "dev-test"

  # 追加とコミット
  npx tsx src/index.ts add README.md -i "doc"
  npx tsx src/index.ts commit -i "commit for test" -c 0.8 --goal "goal" --context "ctx" --impact "low"

  # ログ/表示/ドライラン
  npx tsx src/index.ts log
  npx tsx src/index.ts show <idPrefix>
  npx tsx src/index.ts checkout <idPrefix> -n -A
  ```

---

## デバッグのヒント

- 逐次ログ: `console.log` / `pc.*` を要所で活用。
- インスペクタ: `node --inspect-brk dist/index.cjs ...` または `tsx` 実行時の `--inspect`。
- 出力を短く保つ: 大量の一覧は必要に応じて件数制限や要約を検討。

---

## リリース（暫定）

1) `npm run build`
2) バージョン更新（必要であれば `package.json`）
3) タグ/配布はプロジェクト方針に合わせて実施

---

## 将来拡張のアイデア

- 差分の効率化・バイナリ判定
- 部分チェックアウト/適用（パッチベース）
- メタデータの拡張（レビュー、タスク連携）
- 簡易テストスイートの整備（ユニット/統合）

---

何か不明点や改善提案があれば、Issue あるいはPRで気軽に連絡してください。
