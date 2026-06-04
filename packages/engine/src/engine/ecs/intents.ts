import type { Vec3 as vec3, Vec4 as vec4 } from '../../math/index';
import type { EnvironmentConfig } from '../../types/cmds/environment';
import type { Shadow3dConfig } from '../../types/cmds/shadow';
import type { NotificationLevel } from '../../types/kinds';
import type { JsonObject } from '../../types/json';
import type {
  CameraProps,
  ComponentType,
  GeometryProps,
  LightProps,
  MaterialProps,
  ModelProps,
  Shape2DProps,
  Sprite2DProps,
  TagProps,
  TextureProps,
  TransformProps
} from './components';

export type Intent =
  | { type: 'configure-environment'; config: EnvironmentConfig }
  | {
      type: 'request-resource-list';
      resourceType: 'model' | 'material' | 'texture' | 'geometry' | 'light' | 'camera';
    }
  | { type: 'create-entity'; worldId: number; entityId: number }
  | { type: 'remove-entity'; entityId: number }
  | {
      type: 'update-transform';
      entityId: number;
      props: TransformProps;
    }
  | {
      type: 'attach-camera';
      entityId: number;
      props: CameraProps;
    }
  | {
      type: 'attach-model';
      entityId: number;
      props: ModelProps;
    }
  | {
      type: 'attach-sprite2d';
      entityId: number;
      props: Sprite2DProps;
    }
  | {
      type: 'attach-shape2d';
      entityId: number;
      props: Shape2DProps;
    }
  | {
      type: 'attach-light';
      entityId: number;
      props: LightProps;
    }
  | {
      type: 'attach-tag';
      entityId: number;
      props: TagProps;
    }
  | { type: 'create-material'; resourceId: number; props: MaterialProps }
  | { type: 'create-geometry'; resourceId: number; props: GeometryProps }
  | { type: 'create-texture'; resourceId: number; props: TextureProps }
  | { type: 'dispose-material'; resourceId: number }
  | { type: 'dispose-texture'; resourceId: number }
  | { type: 'dispose-geometry'; resourceId: number }
  | { type: 'detach-component'; entityId: number; componentType: ComponentType }
  | { type: 'set-parent'; entityId: number; parentId: number | null }
  | {
      type: 'send-notification';
      level: NotificationLevel;
      title: string;
      message: string;
    }
  | { type: 'configure-shadows'; config: Shadow3dConfig }
  | {
      type: 'gizmo-draw-line';
      start: vec3;
      end: vec3;
      color: vec4;
      thickness?: number;
    }
  | {
      type: 'gizmo-draw-aabb';
      min: vec3;
      max: vec3;
      color: vec4;
      thickness?: number;
    }
  | {
      type: 'gizmo-draw-polyline';
      points: vec3[];
      color: vec4;
      closed?: boolean;
      thickness?: number;
    }
  | {
      type: 'custom';
      name: string;
      data: JsonObject;
    };

/** Internal World Events. */
export type WorldEvent =
  | { type: 'entity-created'; entityId: number }
  | { type: 'entity-destroyed'; entityId: number }
  | { type: 'component-added'; entityId: number; componentType: ComponentType }
  | {
      type: 'component-removed';
      entityId: number;
      componentType: ComponentType;
    };
