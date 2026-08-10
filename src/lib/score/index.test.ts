import { describe, expect, it } from "vitest";
import { midiNoteName, neckPosition, remapNotes } from "./index";
describe("score remapping", () => {
  const profile = {
    midiMin: 60,
    midiMax: 72,
    calibration: [
      [60, 0],
      [72, 1],
    ] as [number, number][],
  };
  it("matches Rust calibration interpolation golden values", () => {
    expect(neckPosition(profile, 66)).toBe(0.5);
    expect(neckPosition(profile, 73)).toBeNull();
  });
  it("remaps by transpose without changing timing", () => {
    expect(
      remapNotes(
        [
          {
            startSec: 1,
            durationSec: 0.5,
            midiNote: 60,
            centsOffset: 0,
            neckPos: 0,
          },
        ],
        6,
        profile,
      ),
    ).toEqual([
      {
        startSec: 1,
        durationSec: 0.5,
        midiNote: 66,
        centsOffset: 0,
        neckPos: 0.5,
      },
    ]);
  });
  it("names midi notes", () => {
    expect(midiNoteName(69)).toBe("A4");
    expect(midiNoteName(60)).toBe("C4");
    expect(midiNoteName(57)).toBe("A3");
  });
});
