---
name: bug_fix_protocol
description: バグ修正時のリグレッションテスト作成義務と、グローバル状態共有の禁止ルール。
---

# バグ修正プロトコル

> [!CAUTION]
> **テストの Source of Truth は「現在のコード」ではなく「Blueprint（要件仕様）」である。**

## 1. リグレッションテストファースト（絶対ルール）

バグ修正は必ずこの順序で行う：

1. **テストを先に書く**: 修正**前**のコードで**失敗する**テストを書き、失敗を確認
2. **コードを修正する**: テストがパスするまで修正
3. **全テスト実行**: `npm run test` で既存テストへの影響を確認

> [!CAUTION]
> **「修正してからテストを書く」は禁止。** テストがバグを検出できるか保証できなくなる。

## 2. トートロジー（自己正当化）の防止

AIが「現在のコードの挙動」を正解としてテストを書くと、**バグを保護するテスト**が生まれる。

### 絶対ルール
- テストの期待値は `blueprint.md` / `README.md` の仕様から導出する
- 仕様が不明確な場合は、テストを書く前にユーザーに確認する
- 仕様が未定義の挙動を実装する場合は、まず `blueprint.md` に仕様を追記してから実装

## 3. グローバル状態共有の禁止

マルチテナントシステムでは、エンティティIDごとにデータを分離する：

```typescript
// ❌ グローバル変数で全IDの状態を共有
let mockStats = { totalRevenue: 500 };

// ✅ IDごとに独立
let dataStore: Record<string, EntityData> = {};

function getEntityData(entityId: string): EntityData {
  if (!dataStore[entityId]) {
    dataStore[entityId] = createInitialData();
  }
  return dataStore[entityId];
}
```

### テストで検証すべき観点

| 観点 | テスト内容 |
|---|---|
| **データ分離** | entityAへの操作がentityBに影響しない |
| **初期状態** | 新規entityは空データで開始 |
| **永続化整合性** | writeしたデータがreadで正しく返る |
| **リセット** | resetが全entityのデータをクリアする |
| **特殊IDの分岐** | デモ用IDなどの特殊分岐が正しく動作する |
