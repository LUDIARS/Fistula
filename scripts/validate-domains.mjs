// spec/domains のドメイン定義が Anatomia に読める形かを検証する。
//
// このリポジトリの main はまだ足場だけで、アプリケーションコードは別ブランチに
// ある。npm の install / build を登録テストにしても main の実態に対しては
// 成立しないため、main に実在する唯一の検証対象であるドメイン定義をここで
// 確かめる。アプリケーションコードが main に入ったら、この検証は残したまま
// install / build を登録テストへ戻す。

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// カレントディレクトリではなくリポジトリ構造を基準にする。
// CI でも手元でも、どこから起動しても同じ場所を見る。
const DOMAIN_DIR = fileURLToPath(new URL("../spec/domains/", import.meta.url));
const DOMAIN_DIR_LABEL = "spec/domains";

function fail(message) {
  console.error(`NG ${message}`);
  process.exitCode = 1;
}

let files;
try {
  files = readdirSync(DOMAIN_DIR, { withFileTypes: true })
    .filter((e) => e.isFile() && e.name.endsWith(".domain.json"))
    .map((e) => e.name);
} catch (error) {
  fail(`${DOMAIN_DIR_LABEL} を読めない: ${error.message}`);
  process.exit(1);
}

if (files.length === 0) {
  fail(`${DOMAIN_DIR_LABEL} にドメイン定義が 1 本も無い`);
  process.exit(1);
}

// ドメイン名 -> 最初に定義したファイル。重複時にどちらと衝突したかを示す。
const seen = new Map();

for (const file of files) {
  const label = `${DOMAIN_DIR_LABEL}/${file}`;
  let parsed;
  try {
    parsed = JSON.parse(readFileSync(join(DOMAIN_DIR, file), "utf8"));
  } catch (error) {
    fail(`${label} が JSON として読めない: ${error.message}`);
    continue;
  }

  for (const domain of Array.isArray(parsed) ? parsed : [parsed]) {
    if (!domain || typeof domain.name !== "string" || domain.name.length === 0) {
      fail(`${label} に name が無いドメインがある`);
      continue;
    }
    if (seen.has(domain.name)) {
      fail(`${label} のドメイン名 ${domain.name} が ${seen.get(domain.name)} と重複している`);
    }
    seen.set(domain.name, label);

    const membership = domain.membership;
    if (!Array.isArray(membership) || membership.length === 0) {
      fail(`${domain.name} に membership が無い`);
      continue;
    }
    for (const entry of membership) {
      if (!entry || typeof entry.pathPattern !== "string") {
        fail(`${domain.name} の membership に pathPattern が無い項目がある`);
        continue;
      }
      try {
        new RegExp(entry.pathPattern);
      } catch (error) {
        fail(`${domain.name} の pathPattern が正規表現として不正: ${entry.pathPattern}`);
      }
    }
  }
}

if (process.exitCode) {
  console.error(`ドメイン定義の検証に失敗した (${files.length} ファイル)`);
} else {
  console.log(`OK ドメイン定義 ${seen.size} 本 / ${files.length} ファイル`);
}
