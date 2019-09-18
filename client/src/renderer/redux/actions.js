/** @format */
import {
  ADD_COMMAND,
  UPDATE_CARET_POSITION,
  SET_METADATA,
  SET_STYLES,
} from "./actionTypes";

export const addCommand = (id, name, keys, behaviour) => ({
  type: ADD_COMMAND,
  payload: {
    id,
    name,
    keys,
    behaviour,
  },
});

export const updateCaretPos = ({ positionStart, positionEnd }) => ({
  type: UPDATE_CARET_POSITION,
  payload: {
    positionStart,
    positionEnd,
  },
});

export const setMetadata = (key, data) => ({
  type: SET_METADATA,
  key,
  data,
});

export const setStyles = (key, data) => ({
  type: SET_STYLES,
  key,
  data,
});

export const setPageStyles = (key, data) => ({
  type: SET_PAGE_STYLES,
  key,
  data,
});
