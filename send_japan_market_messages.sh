#!/bin/bash

# Japanese market discussion messages (15 messages total)
# Members: super, admin, employee1,3,5,7,9,11
messages=(
  'super:皆さん、おはようございます。日本市場の四半期レポートを共有します。'
  'admin:売上が前年比120%増加しました。特に東京地区の成長が顕著です。'
  'employee3:大阪の新店舗オープンも好調です。初月で目標を15%上回りました。'
  'employee3:現地パートナーとの関係も良好で、来月さらに2店舗展開予定です。'
  'employee5:マーケティング面では、インフルエンサーとのコラボが効果的でした。'
  'super:素晴らしい成果ですね。京都進出の計画はどうなっていますか？'
  'employee7:京都は観光客向けの特別商品を準備中です。地元の伝統工芸とコラボします。'
  'admin:規制面での進展もあります。新しいライセンスが承認されました。'
  'employee9:カスタマーサポートは日本語ネイティブを5名追加採用しました。'
  'employee9:応答時間が平均3分に短縮され、顧客満足度が向上しています。'
  'employee11:競合分析によると、我々の価格戦略は適切です。品質重視が功を奏しています。'
  'super:配送時間の改善についてはどうでしょうか？'
  'employee1:ヤマト運輸との提携により、翌日配送エリアが80%まで拡大しました。'
  'admin:次期の目標は北海道と九州への展開です。地域特性を考慮した戦略が必要です。'
  'super:はい、引き続き頑張りましょう。来週詳細な計画を議論しましょう。'
)

# Array of members who can send messages
members=("super" "admin" "employee1" "employee3" "employee5" "employee7" "employee9" "employee11")

# Send messages
i=0
for msg in "${messages[@]}"; do
  # Parse sender and content
  sender=$(echo "$msg" | cut -d: -f1)
  content=$(echo "$msg" | cut -d: -f2-)
  
  # Get appropriate token
  if [ "$sender" = "super" ]; then
    TOKEN=$(cat super_token.txt)
  elif [ "$sender" = "admin" ]; then
    TOKEN=$(cat admin_token.txt)
  else
    TOKEN=$(cat ${sender}_token.txt)
  fi
  
  # Send message
  curl -s -X POST http://45.77.178.85:8080/api/chat/5/messages \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"content\": \"$content\", \"files\": null}" | jq -r '.data.id'
  
  i=$((i + 1))
  echo "[$i/15] 日本市場: $sender"
  sleep 0.5
done

echo "日本市場チャンネルの討論が完了しました。" 