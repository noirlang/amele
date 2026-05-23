import test from "node:test";
import assert from "node:assert";
import { translations } from "../ui/i18n.js";

test("i18n Translation Integrity Tests", async (t) => {
  await t.test("translations dictionary structure is valid", () => {
    assert.ok(translations, "translations export should be defined");
    assert.ok(translations.tr, "Turkish translation dictionary should be defined");
    assert.ok(translations.en, "English translation dictionary should be defined");
    
    assert.strictEqual(typeof translations.tr, "object", "tr should be an object");
    assert.strictEqual(typeof translations.en, "object", "en should be an object");
  });

  await t.test("compare Turkish and English key coverage", () => {
    const trKeys = Object.keys(translations.tr);
    const enKeys = Object.keys(translations.en);

    const missingInEn = trKeys.filter((key) => !(key in translations.en));
    const missingInTr = enKeys.filter((key) => !(key in translations.tr));

    if (missingInEn.length > 0) {
      console.warn("\n⚠️ Keys present in TR but missing in EN:");
      missingInEn.forEach(key => console.warn(`  - ${key}`));
    }

    if (missingInTr.length > 0) {
      console.warn("\n⚠️ Keys present in EN but missing in TR:");
      missingInTr.forEach(key => console.warn(`  - ${key}`));
    }

    assert.strictEqual(
      missingInEn.length,
      0,
      `There are ${missingInEn.length} keys in TR that are missing in EN.`
    );
    assert.strictEqual(
      missingInTr.length,
      0,
      `There are ${missingInTr.length} keys in EN that are missing in TR.`
    );
  });

  await t.test("verify template placeholder variable matches", () => {
    const trKeys = Object.keys(translations.tr);
    const variableRegex = /\{([a-zA-Z0-9_]+)\}/g;

    let mismatchCount = 0;

    for (const key of trKeys) {
      const trVal = translations.tr[key];
      const enVal = translations.en[key];

      if (typeof trVal !== "string" || typeof enVal !== "string") {
        continue;
      }

      const trMatches = [...trVal.matchAll(variableRegex)].map(m => m[1]).sort();
      const enMatches = [...enVal.matchAll(variableRegex)].map(m => m[1]).sort();

      const diffTr = trMatches.filter(m => !enMatches.includes(m));
      const diffEn = enMatches.filter(m => !trMatches.includes(m));

      if (diffTr.length > 0 || diffEn.length > 0) {
        mismatchCount++;
        console.error(`\n❌ Placeholder mismatch on key: "${key}"`);
        console.error(`  TR [${trVal}]: variables [${trMatches.join(", ")}]`);
        console.error(`  EN [${enVal}]: variables [${enMatches.join(", ")}]`);
      }
    }

    assert.strictEqual(
      mismatchCount,
      0,
      `Found ${mismatchCount} translation keys with mismatched variable placeholders.`
    );
  });
});
