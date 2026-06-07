export const DEMO_IDS = ['001', '002', '003', '004'] as const;

export type DemoId = (typeof DEMO_IDS)[number];
