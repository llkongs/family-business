// Product Database Types for Yellow Wine Store
// Designed with supermarket-style product management attributes

// 商品基础信息
export interface Product {
    // 唯一标识
    id: string;
    sku: string;                    // 商品编码
    barcode: string;                // 条形码 (EAN-13)

    // 基本信息
    name: string;                   // 商品名称
    brand: Brand;                   // 品牌
    category: Category;             // 分类

    // 规格信息
    specification: string;          // 规格 (如: 500ml, 1.5L)
    unit: string;                   // 单位 (瓶、箱、坛)
    packSize: number;               // 包装规格 (1瓶、6瓶/箱)
    weight: number;                 // 净重 (ml 或 g)

    // 价格信息
    retailPrice: number;            // 零售价
    costPrice?: number;             // 成本价
    memberPrice?: number;           // 会员价
    promotionPrice?: number;        // 促销价

    // 库存信息
    stock: number;                  // 库存数量
    safetyStock: number;            // 安全库存
    warehouseLocation?: string;     // 仓位

    // 产品特性
    origin: string;                 // 产地
    shelfLife: number;              // 保质期 (月)
    storageCondition: string;       // 储存条件

    // 酒类特有属性
    alcoholContent: number;         // 酒精度 (%)
    vintage?: number;               // 年份
    brewingProcess: string;         // 酿造工艺
    flavorProfile: string;          // 风味描述
    servingSuggestion?: string;     // 饮用建议

    // 媒体资源
    mainImage: string;              // 主图
    detailImages?: string[];        // 详情图
    video?: string;                 // 视频

    // 描述
    shortDescription: string;       // 简短描述
    longDescription?: string;       // 详细描述

    // 供应商信息
    supplier?: Supplier;

    // 状态
    status: ProductStatus;
    isHot: boolean;                 // 热销
    isNew: boolean;                 // 新品
    isPromotion: boolean;           // 促销中

    // 时间戳
    createdAt: string;
    updatedAt: string;
}

// 品牌
export interface Brand {
    id: string;
    name: string;                   // 品牌名称
    logo?: string;                  // 品牌logo
    story?: string;                 // 品牌故事
    foundedYear?: number;           // 创立年份
    origin?: string;                // 品牌产地
}

// 分类
export interface Category {
    id: string;
    name: string;                   // 分类名称
    parentId?: string;              // 父分类ID
    level: number;                  // 分类层级 (1, 2, 3)
    icon?: string;                  // 分类图标
}

// 供应商
export interface Supplier {
    id: string;
    name: string;                   // 供应商名称
    contact?: string;               // 联系人
    phone?: string;                 // 联系电话
    address?: string;               // 地址
}

// 商品状态
export type ProductStatus = 'active' | 'inactive' | 'outOfStock' | 'discontinued';

// 数据库结构
export interface ProductDatabase {
    version: string;
    lastUpdated: string;
    brands: Brand[];
    categories: Category[];
    suppliers: Supplier[];
    products: Product[];
}
