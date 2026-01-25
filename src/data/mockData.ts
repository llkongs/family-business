// Mock Data for Family Business

export interface StoreInfo {
    name: string;
    phone: string;
    qrCodeUrl: string;
}

export interface MediaItem {
    type: 'video' | 'image';
    url: string;
    title?: string;
    duration?: number; // for images, duration in ms
}

export interface Product {
    id: string;
    name: string;
    description: string;
    price: number;
    image: string;
    categoryId: string;
}

export interface Category {
    id: string;
    name: string;
    icon?: string;
}

// Get base URL for assets (works with GitHub Pages subpath)
const BASE_URL = import.meta.env.BASE_URL;

// Store Information
export const storeInfo: StoreInfo = {
    name: '绍兴黄酒专卖',
    phone: '15936229925',
    qrCodeUrl: `${BASE_URL}images/qrcode.jpg`
};

// Media playlist - videos and images for the ad display
export const mediaPlaylist: MediaItem[] = [
    // 古越龙山系列
    {
        type: 'image',
        url: `${BASE_URL}images/brands/guyuelongshan_palace.png`,
        title: '古越龙山 - 国宴专用',
        duration: 5000
    },
    {
        type: 'image',
        url: `${BASE_URL}images/brands/guyuelongshan_brewing.png`,
        title: '古越龙山 - 传统酿造',
        duration: 5000
    },
    // 会稽山系列
    {
        type: 'image',
        url: `${BASE_URL}images/brands/kuaijishan_scenery.png`,
        title: '会稽山 - 山水意境',
        duration: 5000
    },
    {
        type: 'image',
        url: `${BASE_URL}images/brands/kuaijishan_cellar.png`,
        title: '会稽山 - 280年传承',
        duration: 5000
    },
    // 女儿红系列
    {
        type: 'image',
        url: `${BASE_URL}images/brands/nverhong_bride.png`,
        title: '女儿红 - 婚嫁祝福',
        duration: 5000
    },
    {
        type: 'image',
        url: `${BASE_URL}images/brands/nverhong_tradition.png`,
        title: '女儿红 - 十八年传承',
        duration: 5000
    }
];

// Categories
export const categories: Category[] = [
    { id: 'hot', name: '热销推荐', icon: '🔥' },
    { id: 'staple', name: '主食', icon: '🍚' },
    { id: 'meat', name: '肉类', icon: '🥩' },
    { id: 'seafood', name: '海鲜', icon: '🦐' },
    { id: 'vegetable', name: '时蔬', icon: '🥬' },
    { id: 'soup', name: '汤品', icon: '🍲' },
    { id: 'drink', name: '饮品', icon: '🥤' },
    { id: 'dessert', name: '甜点', icon: '🍰' }
];

// Products
export const products: Product[] = [
    // 热销推荐
    {
        id: '1',
        categoryId: 'hot',
        name: '招牌红烧肉',
        description: '精选五花肉，慢火炖煮2小时，肥而不腻，入口即化。配以秘制酱汁，色泽红亮诱人。',
        price: 48,
        image: 'https://images.unsplash.com/photo-1623653387945-2fd25214f8fc?w=400&h=400&fit=crop'
    },
    {
        id: '2',
        categoryId: 'hot',
        name: '松鼠鳜鱼',
        description: '新鲜鳜鱼现杀现做，外酥里嫩，酸甜可口。传统苏帮菜经典之作。',
        price: 88,
        image: 'https://images.unsplash.com/photo-1534080564583-6be75777b70a?w=400&h=400&fit=crop'
    },
    {
        id: '3',
        categoryId: 'hot',
        name: '蒜蓉粉丝蒸扇贝',
        description: '精选新鲜扇贝，配以龙口粉丝和特制蒜蓉酱，鲜嫩多汁。',
        price: 58,
        image: 'https://images.unsplash.com/photo-1559339352-11d035aa65de?w=400&h=400&fit=crop'
    },
    // 主食
    {
        id: '4',
        categoryId: 'staple',
        name: '扬州炒饭',
        description: '粒粒分明的米饭配以鸡蛋、火腿、青豆、玉米，经典美味。',
        price: 22,
        image: 'https://images.unsplash.com/photo-1603133872878-684f208fb84b?w=400&h=400&fit=crop'
    },
    {
        id: '5',
        categoryId: 'staple',
        name: '手工拉面',
        description: '传统手工拉面，筋道爽滑，配以浓郁牛骨汤底。',
        price: 28,
        image: 'https://images.unsplash.com/photo-1569718212165-3a8278d5f624?w=400&h=400&fit=crop'
    },
    {
        id: '6',
        categoryId: 'staple',
        name: '葱油拌面',
        description: '细面配以香葱油和酱油，简单却回味无穷。',
        price: 18,
        image: 'https://images.unsplash.com/photo-1552611052-33e04de081de?w=400&h=400&fit=crop'
    },
    // 肉类
    {
        id: '7',
        categoryId: 'meat',
        name: '糖醋里脊',
        description: '外酥里嫩，酸甜适中，老少皆宜的经典菜品。',
        price: 38,
        image: 'https://images.unsplash.com/photo-1529692236671-f1f6cf9683ba?w=400&h=400&fit=crop'
    },
    {
        id: '8',
        categoryId: 'meat',
        name: '宫保鸡丁',
        description: '鸡肉嫩滑，花生酥脆，麻辣鲜香，下饭神器。',
        price: 35,
        image: 'https://images.unsplash.com/photo-1525755662778-989d0524087e?w=400&h=400&fit=crop'
    },
    {
        id: '9',
        categoryId: 'meat',
        name: '水煮牛肉',
        description: '川菜经典，牛肉鲜嫩，麻辣过瘾。',
        price: 52,
        image: 'https://images.unsplash.com/photo-1544025162-d76694265947?w=400&h=400&fit=crop'
    },
    // 海鲜
    {
        id: '10',
        categoryId: 'seafood',
        name: '清蒸鲈鱼',
        description: '新鲜鲈鱼清蒸，保留原汁原味，肉质细嫩。',
        price: 68,
        image: 'https://images.unsplash.com/photo-1519708227418-c8fd9a32b7a2?w=400&h=400&fit=crop'
    },
    {
        id: '11',
        categoryId: 'seafood',
        name: '椒盐虾',
        description: '大虾外酥里嫩，椒盐提味，香脆可口。',
        price: 78,
        image: 'https://images.unsplash.com/photo-1565680018434-b513d5e5fd47?w=400&h=400&fit=crop'
    },
    // 时蔬
    {
        id: '12',
        categoryId: 'vegetable',
        name: '蒜蓉西兰花',
        description: '新鲜西兰花配以蒜蓉快炒，清脆爽口。',
        price: 22,
        image: 'https://images.unsplash.com/photo-1459411552884-841db9b3cc2a?w=400&h=400&fit=crop'
    },
    {
        id: '13',
        categoryId: 'vegetable',
        name: '干煸四季豆',
        description: '四季豆煸至微焦，配以肉末干辣椒，香辣酥脆。',
        price: 25,
        image: 'https://images.unsplash.com/photo-1551326844-4df70f78d0e9?w=400&h=400&fit=crop'
    },
    {
        id: '14',
        categoryId: 'vegetable',
        name: '上汤娃娃菜',
        description: '娃娃菜配以浓郁上汤，鲜嫩可口。',
        price: 28,
        image: 'https://images.unsplash.com/photo-1540420773420-3366772f4999?w=400&h=400&fit=crop'
    },
    // 汤品
    {
        id: '15',
        categoryId: 'soup',
        name: '番茄蛋花汤',
        description: '酸甜可口，营养丰富，老少皆宜。',
        price: 15,
        image: 'https://images.unsplash.com/photo-1547592166-23ac45744acd?w=400&h=400&fit=crop'
    },
    {
        id: '16',
        categoryId: 'soup',
        name: '老母鸡汤',
        description: '精选散养老母鸡，慢火炖煮4小时，汤浓味鲜。',
        price: 58,
        image: 'https://images.unsplash.com/photo-1583608205776-bfd35f0d9f83?w=400&h=400&fit=crop'
    },
    // 饮品
    {
        id: '17',
        categoryId: 'drink',
        name: '鲜榨橙汁',
        description: '新鲜现榨，富含维C，清爽解腻。',
        price: 18,
        image: 'https://images.unsplash.com/photo-1621506289937-a8e4df240d0b?w=400&h=400&fit=crop'
    },
    {
        id: '18',
        categoryId: 'drink',
        name: '酸梅汤',
        description: '传统配方，酸甜开胃，消暑解渴。',
        price: 12,
        image: 'https://images.unsplash.com/photo-1499638673689-79a0b5115d87?w=400&h=400&fit=crop'
    },
    // 甜点
    {
        id: '19',
        categoryId: 'dessert',
        name: '芒果西米露',
        description: '香甜芒果配以Q弹西米，清凉甜蜜。',
        price: 22,
        image: 'https://images.unsplash.com/photo-1488477181946-6428a0291777?w=400&h=400&fit=crop'
    },
    {
        id: '20',
        categoryId: 'dessert',
        name: '红豆双皮奶',
        description: '顺滑双皮奶配以甜蜜红豆，经典港式甜品。',
        price: 20,
        image: 'https://images.unsplash.com/photo-1551024506-0bccd828d307?w=400&h=400&fit=crop'
    }
];

// Helper function to get products by category
export function getProductsByCategory(categoryId: string): Product[] {
    return products.filter(p => p.categoryId === categoryId);
}

// Helper function to get product by id
export function getProductById(productId: string): Product | undefined {
    return products.find(p => p.id === productId);
}
