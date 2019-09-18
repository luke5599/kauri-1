/** @format */

import * as actions from "./pageStyles";
import * as types from "./types";

describe("actions", () => {
  it("should create an action to set page styles", () => {
    const key = "page";
    const data = {
      size: "A4",
      orientation: "portrait",
      margin: "2cm 3cm",
      borderWidth: "1px",
      borderStyle: "solid",
    };

    const expectedAction = {
      type: types.SET_PAGE_STYLES,
      key,
      data,
    };
    expect(actions.setPageStyles(key, data)).toEqual(expectedAction);
  });
});
