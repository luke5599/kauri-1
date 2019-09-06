/** @format */
import { combineReducers } from "redux";
import { ADD_COMMAND } from "../actionTypes";

export default combineReducers({
  allIds: allShortcuts,
  byId: shortcutsById,
});

function allShortcuts(state = [], action) {
  switch (action.type) {
    case ADD_COMMAND:
      return [
        ...state,
        ...action.payload.shortcuts.map(shortcut => shortcut.id),
      ];

    default:
      return state;
  }
}

function shortcutsById(state = {}, action) {
  switch (action.type) {
    case ADD_COMMAND:
      const shortcuts = {};

      action.payload.shortcuts.forEach(shortcut => {
        shortcuts[shortcut.id] = {
          ...shortcut,
          commandId: action.payload.id,
        };
      });
      return { ...state, ...shortcuts };

    default:
      return state;
  }
}
