# Engine

- [ ] QLC+がスレッドをどう分けているか調査
- [ ] 構造化ログの導入(log クレート)

# ファンクション

- [ ] (YAGNI 原則により後回し)Fader, Chaser で最後のフレームに StartFunction(next_id)してから次のファンクションが走り出すまでのラグ->Engine::tick を再帰的に(処理順を定める)?

## Fader

- [ ] シーン以外のフェードイン実装
- [ ] Fader が Chaser に依存->Fader に完了後コマンド(callback)を持たせる

# エディターについて

- [ ] 宣言的 UI?
