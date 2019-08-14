/** @format */

import "./styles.scss";

import { h, Component, createRef } from "preact";
import { renderDocumentNodes } from "dom/render";
import { connect, useDispatch, useSelector } from "react-redux";
import { addCommand, updateCaretPos } from "redux/actions";
import ToolBar from "components/Editor/ToolBar";

const POST_URI = "http://127.0.0.1:3000/key";

/**
 * A document editing component.
 * @extends Component
 */
class Editor extends Component {
  /**
   * Constructs a new editor component.
   * @param {Object} props Component properties.
   * @param {DOM} props.dom A DOM to render in the editor.
   */
  constructor(props) {
    super(props);
    this.contentEditableDiv = createRef();
    this.clearContentEditable = this.clearContentEditable.bind(this);
    this.handleDocumentClick = this.handleDocumentClick.bind(this);
  }

  /**
   * @private
   */
  componentDidMount() {
    document.execCommand("defaultParagraphSeparator", false, "p");
  }

  /**
   * Clears the contents of the contenteditable div, designed for use before loading a new file.
   */
  clearContentEditable() {
    this.contentEditableDiv.current.innerHTML = "";
  }

  /**
   * Returns absolute values of caret's start/end positions
   */
  getCaretPos() {
    const range = document.getSelection().getRangeAt(0);
    const preSelectionRange = range.cloneRange();
    preSelectionRange.selectNodeContents(this.contentEditableDiv.current);
    preSelectionRange.setEnd(range.startContainer, range.startOffset);
    const startPosition = preSelectionRange.toString().length;
    const endPosition = startPosition + range.toString().length;
    return { startPosition, endPosition };
  }

  handleDocumentClick()
  {
    return this.props.updateCaretPos(this.getCaretPos());
  }

  render(props) {
    return (
      <div>
        <ToolBar />

        <div
          ref={this.contentEditableDiv}
          class="editor"
          id="editor"
          contenteditable="true"
          onClick={this.handleDocumentClick}
        >
          {renderDocumentNodes(props.dom.children)}
        </div>
      </div>
    );
  }
}

export default connect(
  null,
  { updateCaretPos }
)(Editor);
