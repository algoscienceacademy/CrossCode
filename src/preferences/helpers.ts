import { PreferencePage, PreferenceItem } from "./types";
import { preferenceRegistry } from "./registry";

export function createPreferencePage(
  id: string,
  name: string,
  items: PreferenceItem[],
  options?: {
    description?: string;
    category?: string;
    onLoad?: () => void | Promise<void>;
    onSave?: () => void | Promise<void>;
  }
): PreferencePage {
  const page: PreferencePage = {
    id,
    name,
    items,
    description: options?.description,
    category: options?.category,
    onLoad: options?.onLoad,
    onSave: options?.onSave,
  };

  preferenceRegistry.registerPage(page);
  return page;
}

export function createCustomPreferencePage(
  id: string,
  name: string,
  component: React.ComponentType,
  options?: {
    description?: string;
    category?: string;
    onLoad?: () => void | Promise<void>;
    onSave?: () => void | Promise<void>;
  }
): PreferencePage {
  const page: PreferencePage = {
    id,
    name,
    customComponent: component,
    description: options?.description,
    category: options?.category,
    onLoad: options?.onLoad,
    onSave: options?.onSave,
  };

  preferenceRegistry.registerPage(page);
  return page;
}

export const createItems = {
  text: (
    id: string,
    name: string,
    description?: string,
    defaultValue?: string
  ): PreferenceItem => ({
    id,
    name,
    description,
    type: "text",
    defaultValue,
  }),

  select: (
    id: string,
    name: string,
    options: Array<{ label: string; value: string }>,
    description?: string,
    defaultValue?: string
  ): PreferenceItem => ({
    id,
    name,
    description,
    type: "select",
    options,
    defaultValue,
  }),

  checkbox: (
    id: string,
    name: string,
    description?: string,
    defaultValue?: boolean
  ): PreferenceItem => ({
    id,
    name,
    description,
    type: "checkbox",
    defaultValue,
  }),

  button: (
    id: string,
    name: string,
    description: string,
    onClick: () => void | Promise<void>
  ): PreferenceItem => ({
    id,
    name,
    description,
    type: "button",
    onChange: onClick,
  }),

  number: (
    id: string,
    name: string,
    description?: string,
    defaultValue?: number
  ): PreferenceItem => ({
    id,
    name,
    description,
    type: "number",
    defaultValue,
  }),
};
