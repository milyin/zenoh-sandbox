import type { InjectionKey } from 'vue'

/**
 * Common interface for section group contexts.
 * Both SectionTabGroup and SectionListGroup implement this interface.
 */
export interface SectionGroupContext {
  /** Type of group: 'tab' for tabbed groups, 'list' for vertical list groups */
  type: 'tab' | 'list'
  /** Register a section with the group */
  registerSection: (id: string, title: string, icon: string) => void
  /** Unregister a section from the group */
  unregisterSection: (id: string) => void
  /** Check if a section is currently visible/active */
  isSectionVisible: (id: string) => boolean
}

export const SECTION_GROUP_KEY: InjectionKey<SectionGroupContext> = Symbol('SectionGroup')
