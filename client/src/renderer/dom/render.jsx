/** @format */

import { h } from "preact";
import { convertToPixels } from "helpers/units";
import RenderError from "dom/RenderError";

import * as Nodes from "components/Editor/Nodes";
import * as Elements from "components/Editor/Elements";
import Page from "components/Editor/Page";

/**
 * A map of node types to components.
 *
 * It's faster to perform a lookup in an object when you know the key, than
 * create a giant switch case statement with each possible Node type. Having a
 * lookup object also allows us to create additional nodes at runtime. This
 * could prove valuable once we begin supporting third-party extensions.
 *
 * @type {Object}
 */
const NODE_MAP = Object.freeze({
  caption: Elements.Caption,
  code: Elements.InlineCode,
  codeblock: Elements.CodeBlock,
  heading: Elements.Heading,
  hint: Elements.Hint,
  hyperlink: Elements.Hyperlink,
  linebreak: Nodes.LineBreak,
  list: Elements.List,
  listitem: Elements.ListItem,
  pagebreak: Nodes.PageBreak,
  paragraph: Elements.Paragraph,
  span: Elements.Span,
  table: Elements.Table,
  tablecell: Elements.TableCell,
  tablerow: Elements.TableRow,
  text: Nodes.Text,
});

/**
 * Renders nodes and wraps them across pages as needed.
 * @param {Object[]} nodes An array of document nodes.
 * @return {Component[]} An array of page components.
 */
export function renderPaginatedDocument(nodes) {
  const pages = [];
  const workingHeight = convertToPixels(150);
  let currentHeight = 0;
  let currentPage = [];
  console.log("Working height", workingHeight, "px");

  nodes.forEach(node => {
    const rendered = renderNode(node);
    currentPage.push(rendered);

    pages.push(currentPage);
    console.log(rendered);
    currentPage = [];
  });

  const element = document.createElement("p");
  element.appendChild(document.createTextNode("Hello world"));
  console.log("document.createElement", element);
  console.log("Bounding client rect", element.getBoundingClientRect());

  const x = h("x");
  console.log("X", x);

  return pages.map(page => <Page children={page} />);
}

/**
 * Renders a list of KDF nodes.
 * @param {Object[]} nodes An array of KDF nodes.
 * @return {Component[]} An array of Preact components.
 */
export function renderNodeList(nodes = []) {
  return nodes.map(renderNode);
}

/**
 * Renders a KDF node.
 * @param {Object} node KDF node to render.
 * @return {Component} A rendered Preact component.
 */
export function renderNode(node) {
  // Handle text node shorthand
  if (typeof node === "string") {
    return node;
  }

  const type = node.type.toLowerCase();

  // Handle unknown node type
  if (!(type in NODE_MAP)) {
    throw new RenderError(`Unknown element type '${node.type}'.`);
  }

  // Create and return tag
  const Node = NODE_MAP[type];
  return <Node {...node} />;
}

/**
 * Turns a KCSS styles object into a CSS styles object.
 * @param {Object} styles KCSS styles to render.
 * @return {Object} CSS styles.
 */
export function renderStyles(styles) {
  return styles;
}
