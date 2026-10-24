-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Accounting" (
    "AccountingId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "BranchId" text,
    "BranchName" text,
    "MonetaryCategory" text,
    "Expense" numeric NOT NULL,
    "Income" numeric NOT NULL,
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Accounting_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("AccountingId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."AlexLogOrder" (
    "AlexLogOrderId" varchar(50) NOT NULL,
    "OHID" varchar(50),
    "OrderNumber" varchar(40),
    "Category" varchar(50),
    "Action" varchar(50),
    "SubAction" varchar(50),
    "WhoTakeAction" varchar(50),
    "Remark1" varchar(100),
    "Remark2" varchar(100),
    "CreateBy" varchar(50),
    "CreateDateTime" timestamp NOT NULL,
    "ShopInfoId" text,
    "BRCHID" text,
    "ShopName" text,
    PRIMARY KEY ("AlexLogOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."AlexLogProduct" (
    "AlexLogProductId" varchar(50) NOT NULL,
    "PGID" varchar(50),
    "PIID" varchar(50),
    "Code" varchar(40),
    "Category" varchar(50),
    "Action" varchar(50),
    "SubAction" varchar(50),
    "WhoTakeAction" varchar(50),
    "Remark1" varchar(100),
    "Remark2" varchar(100),
    "CreateBy" varchar(50),
    "CreateDateTime" timestamp NOT NULL,
    "ShopInfoId" text,
    "ShopName" text,
    PRIMARY KEY ("AlexLogProductId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."AlexLogPromotion" (
    "AlexLogPromotionId" varchar(50) NOT NULL,
    "OHID" varchar(50),
    "BRCHID" varchar(50),
    "CustomerName" varchar(50),
    "CustomerPhone" varchar(50),
    "OrderNumber" varchar(40),
    "Category" varchar(50),
    "Action" varchar(50),
    "SubAction" varchar(50),
    "WhoTakeAction" varchar(50),
    "Remark1" varchar(100),
    "Remark2" varchar(100),
    "CreateBy" varchar(50),
    "CreateDateTime" timestamp NOT NULL,
    "ShopInfoId" text,
    "ShopName" text,
    PRIMARY KEY ("AlexLogPromotionId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."AlexLogSaleOrder" (
    "AlexLogSaleOrderId" varchar(50) NOT NULL,
    "SOID" varchar(50),
    "BRCHID" text,
    "OrderNumber" varchar(40),
    "Category" varchar(50),
    "Action" varchar(50),
    "SubAction" varchar(50),
    "WhoTakeAction" varchar(50),
    "Remark1" varchar(100),
    "Remark2" varchar(100),
    "CreateBy" varchar(50),
    "CreateDateTime" timestamp NOT NULL,
    "ShopInfoId" text,
    "ShopName" text,
    PRIMARY KEY ("AlexLogSaleOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."AlexLogUserLogin" (
    "AlexLogUserLoginId" varchar(50) NOT NULL,
    "Code" varchar(40),
    "Category" varchar(50),
    "Action" varchar(50),
    "SubAction" varchar(50),
    "WhoTakeAction" varchar(50),
    "Remark1" varchar(100),
    "Remark2" varchar(100),
    "CreateBy" varchar(50),
    "CreateDateTime" timestamp NOT NULL,
    "ShopInfoId" text,
    "ShopName" text,
    PRIMARY KEY ("AlexLogUserLoginId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Blog" (
    "BlogId" varchar(50) NOT NULL,
    "MainImage" varchar(100),
    "image1" varchar(100),
    "image2" varchar(100),
    "image3" varchar(100),
    "image4" varchar(100),
    "image5" text,
    "image6" text,
    "image7" text,
    "image8" text,
    "image9" text,
    "Title" varchar(100),
    "BlogContent" varchar(2500),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Blog_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("BlogId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."BlogImage" (
    "BlogImageId" varchar(50) NOT NULL,
    "BlogId" varchar(50),
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "ProductItemId" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("BlogImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Borrow" (
    "BorrowId" varchar(50) NOT NULL,
    "BorrowNumber" varchar(50),
    "BorrowOrder" varchar(50),
    "Flag1" varchar(30),
    "Flag2" varchar(30),
    "ProductItemId" varchar(50),
    "Approve" int4 NOT NULL,
    "FromBranch" varchar(50),
    "ToBranch" varchar(50),
    "StartDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "Qty" int4 NOT NULL,
    "PrepareStatus" varchar(50),
    "PrepareDate" timestamp NOT NULL,
    "PrepareBy" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Borrow_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("BorrowId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Branch" (
    "BranchId" varchar(50) NOT NULL,
    "BranchName" varchar(100),
    "Code" varchar(50),
    "Address" varchar(200),
    "WorkingTime" varchar(100),
    "Phone" varchar(50),
    "Description" varchar(50),
    "RunningOrder" int4 NOT NULL,
    "RunningBorrow" int4 NOT NULL,
    "Sticker" int4 NOT NULL,
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "Flag1" varchar(20),
    "Flag2" varchar(20),
    "UpdateDateTime" timestamp NOT NULL,
    "BorrowUpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "QrToken" varchar(50),
    CONSTRAINT "FK_Branch_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("BranchId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Category" (
    "CategoryId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Category_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("CategoryId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Color" (
    "ColorId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Color_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("ColorId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Customer" (
    "CustomerId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Name" varchar(100),
    "Description" varchar(50),
    "Address" varchar(500),
    "Phone" varchar(20),
    "AccountNumber" varchar(100),
    "AccountName" varchar(100),
    "BankName" varchar(100),
    "LastOrder" varchar(20),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "BirthDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "HowToKnowShop" varchar(100),
    "LineId" varchar(50),
    CONSTRAINT "FK_Customer_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("CustomerId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."CustomerPoint" (
    "CustomerPointId" varchar(50) NOT NULL,
    "ShopInfoId" text,
    "CustomerId" varchar(50),
    "SaleOrderId" varchar(50),
    "OrderHeaderId" varchar(50),
    "GetPoint" int4 NOT NULL,
    "UsePoint" int4 NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("CustomerPointId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."CustomerVoucher" (
    "CustomerVoucherId" varchar(50) NOT NULL,
    "CustomerId" varchar(50),
    "VoucherId" varchar(50),
    "SaleOrderItemId" varchar(50),
    "ShopInfoId" varchar(50),
    "Name" varchar(100),
    "Code" varchar(40),
    "Month" int4 NOT NULL,
    "Price" numeric NOT NULL,
    "Value" numeric NOT NULL,
    "Remain" numeric NOT NULL,
    "ExpireDate" timestamp NOT NULL,
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_CustomerVoucher_Customer_CustomerId" FOREIGN KEY ("CustomerId") REFERENCES "public"."Customer"("CustomerId") ON DELETE RESTRICT,
    CONSTRAINT "FK_CustomerVoucher_SaleOrderItem_SaleOrderItemId" FOREIGN KEY ("SaleOrderItemId") REFERENCES "public"."SaleOrderItem"("SaleOrderItemId") ON DELETE RESTRICT,
    CONSTRAINT "FK_CustomerVoucher_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("CustomerVoucherId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."CustomerVoucherLog" (
    "CustomerVoucherLogId" varchar(50) NOT NULL,
    "CustomerId" varchar(50),
    "ShopInfoId" varchar(50),
    "CustomerVoucherId" varchar(50),
    "SaleOrderId" varchar(50),
    "OrderHeaderId" varchar(50),
    "Value" numeric NOT NULL,
    "StateCancel" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_CustomerVoucherLog_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_CustomerVoucherLog_CustomerVoucher_CustomerVoucherId" FOREIGN KEY ("CustomerVoucherId") REFERENCES "public"."CustomerVoucher"("CustomerVoucherId") ON DELETE RESTRICT,
    PRIMARY KEY ("CustomerVoucherLogId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."DeliveryChannel" (
    "DeliveryChannelId" varchar(50) NOT NULL,
    "DeliveryMethod" varchar(100),
    "ShopInfoId" varchar(50),
    "BranchId" text,
    "Status" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_DeliveryChannel_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("DeliveryChannelId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."DiscountSetting" (
    "DiscountSettingId" varchar(50) NOT NULL,
    "Code" varchar(20),
    "Description" varchar(50),
    "StartAmount" numeric NOT NULL,
    "UntilAmount" numeric NOT NULL,
    "Discount" numeric NOT NULL,
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_DiscountSetting_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("DiscountSettingId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."FreeItemSetting" (
    "FreeItemSettingId" varchar(50) NOT NULL,
    "Code" varchar(20),
    "Description" varchar(50),
    "RentQty" numeric NOT NULL,
    "FreeQty" numeric NOT NULL,
    "State" bool NOT NULL,
    "ExcludeCategories" _text,
    "ExcludeTextures" _text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_FreeItemSetting_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("FreeItemSettingId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."InitCode_Category" (
    "InitCode_CategoryId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("InitCode_CategoryId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."InitCode_Color" (
    "InitCode_ColorId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("InitCode_ColorId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."InitCode_MoneyCode" (
    "InitCode_MoneyCodeId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "CodeType" varchar(50),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("InitCode_MoneyCodeId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."InitCode_Size" (
    "InitCode_SizeId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("InitCode_SizeId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."InitCode_Texture" (
    "InitCode_TextureId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("InitCode_TextureId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ItemImage" (
    "ItemImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "ProductItemId" varchar(50),
    "ProductGroupId" varchar(50),
    "Color" varchar(20),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_ItemImage_ProductGroup_ProductGroupId" FOREIGN KEY ("ProductGroupId") REFERENCES "public"."ProductGroup"("ProductGroupId") ON DELETE RESTRICT,
    CONSTRAINT "FK_ItemImage_ProductItem_ProductItemId" FOREIGN KEY ("ProductItemId") REFERENCES "public"."ProductItem"("ProductItemId") ON DELETE RESTRICT,
    PRIMARY KEY ("ItemImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."LogApp" (
    "LogAppId" varchar(50) NOT NULL,
    "Code" varchar(40),
    "Name" varchar(100),
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "ShopInfoId" text,
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "UserImage" text,
    "Username" text,
    "LogAction" text,
    "Remark" text,
    "OrderHeaderId" text,
    "OrderNumber" text,
    "CustomerName" text,
    "InCharge" text,
    "StatusFrom" text,
    "StatusTo" text,
    "Category" text,
    "Amount" numeric NOT NULL,
    "BranchId" varchar(50),
    "PaymentChannel" text,
    "PaymentMethod" text,
    "SaleOrderId" text,
    CONSTRAINT "FK_LogApp_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    PRIMARY KEY ("LogAppId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."MoneyCode" (
    "MoneyCodeId" varchar(50) NOT NULL,
    "Code" varchar(100),
    "CodeType" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_MoneyCode_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("MoneyCodeId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."MultiplePrice" (
    "MultiplePriceId" varchar(50) NOT NULL,
    "ProductItemId" varchar(50),
    "ProductItemCode" varchar(40),
    "ProductGroupId" varchar(50),
    "RentDurationId" varchar(50),
    "Color" varchar(50),
    "Size" varchar(10),
    "Price" numeric NOT NULL,
    "ShopInfoId" varchar(50),
    "BranchId" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_MultiplePrice_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("MultiplePriceId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."OrderDetail" (
    "OrderDetailId" varchar(50) NOT NULL,
    "ProductItemId" varchar(50),
    "OrderHeaderId" varchar(50),
    "BranchId" varchar(50),
    "Code" varchar(40),
    "Flag1" varchar(20),
    "Flag2" varchar(20),
    "State" bool NOT NULL,
    "ReservedQuantity" int4 NOT NULL,
    "RentalPrice" numeric NOT NULL,
    "Bail" numeric NOT NULL,
    "StartDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "MainImage" text,
    "Name" varchar(100),
    "FreeItemDiscount" numeric NOT NULL DEFAULT 0.0,
    "FreeItemQuantity" int4 NOT NULL DEFAULT 0,
    "OrderDetailStatus" varchar(50),
    "WnfOrderId" varchar(50),
    "WnfReceivingDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "WnfRemark" varchar(500),
    CONSTRAINT "FK_OrderDetail_OrderHeader_OrderHeaderId" FOREIGN KEY ("OrderHeaderId") REFERENCES "public"."OrderHeader"("OrderHeaderId") ON DELETE RESTRICT,
    CONSTRAINT "FK_OrderDetail_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_OrderDetail_WnfOrder_WnfOrderId" FOREIGN KEY ("WnfOrderId") REFERENCES "public"."WnfOrder"("WnfOrderId") ON DELETE RESTRICT,
    PRIMARY KEY ("OrderDetailId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."OrderDetailWeb" (
    "OrderDetailWebId" varchar(50) NOT NULL,
    "Name" varchar(100),
    "MainImage" text,
    "ProductItemId" varchar(50),
    "OrderHeaderId" varchar(50),
    "OrderHeaderWebId" varchar(50),
    "BranchId" varchar(50),
    "Code" varchar(30),
    "Flag1" varchar(20),
    "Flag2" varchar(20),
    "State" bool NOT NULL,
    "ReservedQuantity" int4 NOT NULL,
    "RentalPrice" numeric NOT NULL,
    "Bail" numeric NOT NULL,
    "StartDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_OrderDetailWeb_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("OrderDetailWebId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."OrderHeader" (
    "OrderHeaderId" varchar(50) NOT NULL,
    "OrderRef" varchar(50),
    "OrderNumber" varchar(20),
    "OrderNumberRef" varchar(20),
    "BranchId" varchar(50),
    "OrderStatus" varchar(50),
    "Remark1" varchar(1000),
    "Remark2" varchar(700),
    "Remark3" varchar(500),
    "Remark4" varchar(100),
    "CustomerId" varchar(50),
    "CustomerName" varchar(100),
    "CustomerAddress" varchar(500),
    "CustomerPhone" varchar(30),
    "AccountNumber" varchar(100),
    "AccountName" varchar(100),
    "BankName" varchar(100),
    "TransferContact" varchar(100),
    "ReceivingMethod" text,
    "PickupDate" timestamp NOT NULL,
    "ReturnDate" timestamp NOT NULL,
    "IssuedDate" timestamp NOT NULL,
    "paymentMethod" text,
    "paymentDate" timestamp NOT NULL,
    "receivedDate" timestamp NOT NULL,
    "receivedBy" text,
    "bringBackDate" timestamp NOT NULL,
    "bringBackBy" text,
    "PrepareStatus" varchar(50),
    "PrepareDate" timestamp NOT NULL,
    "PrepareBy" text,
    "RentalPriceTotal" numeric NOT NULL,
    "BailTotal" numeric NOT NULL,
    "GrandTotal" numeric NOT NULL,
    "RentalDiscount" numeric NOT NULL,
    "BailDiscount" numeric NOT NULL,
    "PromotionDiscount" numeric NOT NULL,
    "Deduction" numeric NOT NULL,
    "DeductionDate" timestamp NOT NULL,
    "BailReturningAmount" numeric NOT NULL,
    "BailReturningStatus" text,
    "BailReturningDate" timestamp NOT NULL,
    "Description" varchar(500),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "RentalPaymentMethod" varchar(100),
    "BailPaymentDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "BailPaymentMethod" varchar(100),
    "RentalPaymentDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "BailPaymentChannel" varchar(100),
    "RentalPaymentChannel" varchar(100),
    "ReturnChannel" varchar(100),
    "ReturnMethod" varchar(100),
    "BeforeVat" numeric NOT NULL DEFAULT 0.0,
    "InVat" bool NOT NULL DEFAULT false,
    "NetPrice" numeric NOT NULL DEFAULT 0.0,
    "Vat" numeric NOT NULL DEFAULT 0.0,
    "BailNetTotal" numeric NOT NULL DEFAULT 0.0,
    "GetPoint" int4 NOT NULL DEFAULT 0,
    "PointDiscount" numeric NOT NULL DEFAULT 0.0,
    "TaxRate" numeric NOT NULL DEFAULT 0.0,
    "VatSystem" bool NOT NULL DEFAULT false,
    "VoucherDiscount" numeric NOT NULL DEFAULT 0.0,
    CONSTRAINT "FK_OrderHeader_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_OrderHeader_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    PRIMARY KEY ("OrderHeaderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."OrderHeaderWeb" (
    "OrderHeaderWebId" varchar(50) NOT NULL,
    "OrderRef" varchar(50),
    "OrderNumber" varchar(20),
    "OrderNumberRef" varchar(20),
    "BranchId" varchar(50),
    "OrderStatus" varchar(50),
    "Remark1" varchar(1000),
    "Remark2" varchar(700),
    "Remark3" varchar(500),
    "Remark4" varchar(100),
    "CustomerId" varchar(50),
    "CustomerName" varchar(100),
    "CustomerAddress" varchar(500),
    "CustomerPhone" varchar(30),
    "AccountNumber" varchar(100),
    "AccountName" varchar(100),
    "BankName" varchar(100),
    "TransferContact" varchar(100),
    "ReceivingMethod" text,
    "PickupDate" timestamp NOT NULL,
    "ReturnDate" timestamp NOT NULL,
    "IssuedDate" timestamp NOT NULL,
    "paymentMethod" text,
    "paymentDate" timestamp NOT NULL,
    "receivedDate" timestamp NOT NULL,
    "receivedBy" text,
    "bringBackDate" timestamp NOT NULL,
    "bringBackBy" text,
    "PrepareStatus" varchar(50),
    "PrepareDate" timestamp NOT NULL,
    "PrepareBy" text,
    "RentalPriceTotal" numeric NOT NULL,
    "BailTotal" numeric NOT NULL,
    "GrandTotal" numeric NOT NULL,
    "RentalDiscount" numeric NOT NULL,
    "BailDiscount" numeric NOT NULL,
    "PromotionDiscount" numeric NOT NULL,
    "Deduction" numeric NOT NULL,
    "DeductionDate" timestamp NOT NULL,
    "BailReturningAmount" numeric NOT NULL,
    "BailReturningStatus" text,
    "BailReturningDate" timestamp NOT NULL,
    "Description" varchar(500),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "BailPaymentDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "BailPaymentMethod" text,
    "RentalPaymentDate" timestamp NOT NULL DEFAULT '0001-01-01 00:00:00'::timestamp without time zone,
    "RentalPaymentMethod" text,
    "BailPaymentChannel" text,
    "RentalPaymentChannel" text,
    CONSTRAINT "FK_OrderHeaderWeb_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("OrderHeaderWebId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."OrderLog" (
    "OrderLogId" varchar(50) NOT NULL,
    "Code" varchar(40),
    "Name" varchar(100),
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "ShopInfoId" text,
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "OrderNumber" text,
    "LogAction" text,
    "OrderHeaderId" varchar(50),
    "CustomerName" text,
    "InCharge" text,
    "StatusFrom" text,
    "StatusTo" text,
    "Payment" text,
    "Category" text,
    "Amount" numeric NOT NULL,
    CONSTRAINT "FK_OrderLog_OrderHeader_OrderHeaderId" FOREIGN KEY ("OrderHeaderId") REFERENCES "public"."OrderHeader"("OrderHeaderId") ON DELETE RESTRICT,
    PRIMARY KEY ("OrderLogId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."PaymentChannel" (
    "PaymentChannelId" varchar(50) NOT NULL,
    "AccountCode" varchar(100),
    "AccountName" varchar(100),
    "ShopInfoId" varchar(50),
    "BranchId" text,
    "Status" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    CONSTRAINT "FK_PaymentChannel_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("PaymentChannelId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."PickUpImage" (
    "PickUpImageId" varchar(50) NOT NULL,
    "OrderHeaderId" varchar(50),
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" text,
    PRIMARY KEY ("PickUpImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."PointSetting" (
    "PointSettingId" varchar(50) NOT NULL,
    "ShopInfoId" text,
    "Buy" numeric NOT NULL,
    "Point" int4 NOT NULL,
    "Active" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("PointSettingId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ProductGroup" (
    "ProductGroupId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "Name" varchar(100),
    "Code" varchar(40),
    "Category" varchar(10),
    "Texture" varchar(10),
    "Color" varchar(100),
    "Size" varchar(100),
    "Yearth" int4 NOT NULL,
    "Gender" varchar(10),
    "InStock" int4 NOT NULL,
    "RentalPrice" numeric NOT NULL,
    "Bail" numeric NOT NULL,
    "ItemStatus" varchar(40),
    "WebCategory" varchar(500),
    "WebHotItem" text,
    "WebItem" text,
    "MainImage" text,
    "BarcodeImage" text,
    "ItemCode3Digit" text,
    "Description" varchar(255),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "NewRecord" bool NOT NULL,
    "Temporary" bool NOT NULL,
    "RentalStat" int4 NOT NULL,
    "IntendToChange" bool NOT NULL,
    "WebDescription" varchar(800),
    "Group" text,
    "GroupId" text,
    "Recommended" text,
    "WebNewArrival" text,
    "Deleted" bool NOT NULL DEFAULT false,
    CONSTRAINT "FK_ProductGroup_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("ProductGroupId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ProductItem" (
    "ProductItemId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "ProductGroupId" varchar(50),
    "Name" varchar(100),
    "Code" varchar(40),
    "Category" varchar(10),
    "Texture" varchar(10),
    "Size" varchar(10),
    "Yearth" int4 NOT NULL,
    "Gender" varchar(10),
    "InStock" int4 NOT NULL,
    "RentalPrice" numeric NOT NULL,
    "Bail" numeric NOT NULL,
    "ItemStatus" varchar(40),
    "WebCategory" varchar(500),
    "WebHotItem" text,
    "WebItem" text,
    "MainImage" text,
    "BarcodeImage" text,
    "ItemCode3Digit" text,
    "Description" varchar(255),
    "Color" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "NewRecord" bool NOT NULL,
    "Temporary" bool NOT NULL,
    "RentalStat" int4 NOT NULL,
    "IntendToChange" bool NOT NULL,
    "WebDescription" varchar(800),
    "Group" text,
    "GroupId" text,
    "Deleted" bool NOT NULL DEFAULT false,
    CONSTRAINT "FK_ProductItem_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_ProductItem_ProductGroup_ProductGroupId" FOREIGN KEY ("ProductGroupId") REFERENCES "public"."ProductGroup"("ProductGroupId") ON DELETE RESTRICT,
    PRIMARY KEY ("ProductItemId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Promotion" (
    "PromotionId" varchar(50) NOT NULL,
    "Code" varchar(20),
    "StartAmount" numeric NOT NULL,
    "UntilAmount" numeric NOT NULL,
    "Discount" numeric NOT NULL,
    "Description" varchar(50),
    "State" bool NOT NULL,
    "StateDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Promotion_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("PromotionId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."RegisterToken" (
    "RegisterTokenId" varchar(50) NOT NULL,
    "Code" varchar(40),
    "Name" varchar(100),
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "ShopInfoId" text,
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "Token" varchar(70),
    "Flag1" varchar(10),
    "Flag2" varchar(10),
    "EffectiveDateTime" timestamp NOT NULL,
    "ExpireDateTime" timestamp NOT NULL,
    PRIMARY KEY ("RegisterTokenId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."RentalOrder" (
    "RentalOrderId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_RentalOrder_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("RentalOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."RentDuration" (
    "RentDurationId" varchar(50) NOT NULL,
    "RentDays" int4 NOT NULL,
    "ShopInfoId" text,
    "BranchId" text,
    "Status" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("RentDurationId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ReviewImage" (
    "ReviewImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "ProductItemId" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("ReviewImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Role" (
    "RoleId" text NOT NULL,
    "RoleName" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("RoleId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SaleOrder" (
    "SaleOrderId" varchar(50) NOT NULL,
    "OrderNumber" varchar(50),
    "ShopInfoId" varchar(50),
    "CustomerId" varchar(50),
    "CustomerName" varchar(100),
    "CustomerAddress" varchar(500),
    "CustomerPhone" varchar(30),
    "AccountNumber" varchar(100),
    "AccountName" varchar(100),
    "BankName" varchar(100),
    "PaymentMethod" text,
    "Paid" bool NOT NULL,
    "PaymentDate" timestamp NOT NULL,
    "VatSystem" bool NOT NULL,
    "InVat" bool NOT NULL,
    "TaxRate" numeric NOT NULL,
    "PriceSubTotal" numeric NOT NULL,
    "DiscountBeforeVat" numeric NOT NULL,
    "PriceBeforeVat" numeric NOT NULL,
    "Vat" numeric NOT NULL,
    "PriceTotal" numeric NOT NULL,
    "Discount" numeric NOT NULL,
    "PriceGrandTotal" numeric NOT NULL,
    "GetPoint" int4 NOT NULL,
    "Remark" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "Cancel" bool NOT NULL,
    "CancelDateTime" timestamp NOT NULL,
    "CancelBy" varchar(40),
    CONSTRAINT "FK_SaleOrder_Customer_CustomerId" FOREIGN KEY ("CustomerId") REFERENCES "public"."Customer"("CustomerId") ON DELETE RESTRICT,
    CONSTRAINT "FK_SaleOrder_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("SaleOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SaleOrderItem" (
    "SaleOrderItemId" varchar(50) NOT NULL,
    "ShopInfoId" text,
    "SaleOrderId" varchar(50),
    "SaleProductId" varchar(50),
    "VoucherId" varchar(50),
    "Price" numeric NOT NULL,
    "Qty" int4 NOT NULL,
    "TotalPrice" numeric NOT NULL,
    "Name" varchar(100),
    "Code" varchar(40),
    "Size" varchar(10),
    "Color" varchar(50),
    "Month" int4 NOT NULL,
    "Value" numeric NOT NULL,
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "Cancel" bool NOT NULL,
    "CancelDateTime" timestamp NOT NULL,
    "CancelBy" varchar(40),
    CONSTRAINT "FK_SaleOrderItem_SaleOrder_SaleOrderId" FOREIGN KEY ("SaleOrderId") REFERENCES "public"."SaleOrder"("SaleOrderId") ON DELETE RESTRICT,
    CONSTRAINT "FK_SaleOrderItem_SaleProduct_SaleProductId" FOREIGN KEY ("SaleProductId") REFERENCES "public"."SaleProduct"("SaleProductId") ON DELETE RESTRICT,
    CONSTRAINT "FK_SaleOrderItem_Voucher_VoucherId" FOREIGN KEY ("VoucherId") REFERENCES "public"."Voucher"("VoucherId") ON DELETE RESTRICT,
    PRIMARY KEY ("SaleOrderItemId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SaleProduct" (
    "SaleProductId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "Name" varchar(100),
    "Code" varchar(40),
    "Category" varchar(10),
    "Texture" varchar(10),
    "Size" varchar(10),
    "Color" varchar(50),
    "Gender" varchar(10),
    "InStock" int4 NOT NULL,
    "Price" numeric NOT NULL,
    "MainImageId" varchar(50),
    "Description" varchar(255),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "Deleted" bool NOT NULL,
    "DeleteDateTime" timestamp NOT NULL,
    "DeleteBy" varchar(40),
    CONSTRAINT "FK_SaleProduct_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("SaleProductId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SaleProductImage" (
    "SaleProductImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "SaleProductId" varchar(50) NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" text,
    CONSTRAINT "FK_SaleProductImage_SaleProduct_SaleProductId" FOREIGN KEY ("SaleProductId") REFERENCES "public"."SaleProduct"("SaleProductId") ON DELETE CASCADE,
    PRIMARY KEY ("SaleProductImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ShopImage" (
    "ShopImageId" varchar(50) NOT NULL,
    "Name" varchar(100),
    "Code" varchar(40),
    "Seq" varchar(40),
    "Description" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "FileName" varchar(255) NOT NULL DEFAULT ''::character varying,
    "FileType" varchar(255) NOT NULL DEFAULT ''::character varying,
    "PathName" varchar(255) NOT NULL DEFAULT ''::character varying,
    "RelatedId" varchar(40),
    "TableName" varchar(200),
    "UploadFileName" varchar(255) NOT NULL DEFAULT ''::character varying,
    PRIMARY KEY ("ShopImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ShopInfo" (
    "ShopInfoId" varchar(50) NOT NULL,
    "Name" varchar(100),
    "Code" varchar(40),
    "Phone" varchar(50),
    "Line" varchar(50),
    "RunningNumber" varchar(10),
    "NewSeason" varchar(20),
    "DateOfNewSeason" varchar(5),
    "MonthOfNewSeason" varchar(5),
    "FirstYear" varchar(10),
    "Yearth" varchar(5),
    "Description" varchar(50),
    "TermAndCondition" varchar(1200),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopCategory" varchar(100),
    "Age" varchar(20),
    "BranchNumber" varchar(20),
    "Logo" varchar(200),
    "Package" varchar(40),
    "BranchRunning" varchar(10),
    "SaleOrderRunningNumber" int4 NOT NULL DEFAULT 0,
    "WknRunningNumber" int4 NOT NULL DEFAULT 0,
    "StatusInfo" varchar(50),
    PRIMARY KEY ("ShopInfoId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ShopPackage" (
    "ShopPackageId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "Package" varchar(40),
    "StartDateTime" timestamp NOT NULL,
    "EndDateTime" timestamp NOT NULL,
    "Description" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "State" bool NOT NULL,
    "UpdateBy" varchar(40),
    "Paid" bool NOT NULL DEFAULT false,
    "NumberOfUse" int4 NOT NULL DEFAULT 0,
    "Current" bool NOT NULL DEFAULT false,
    PRIMARY KEY ("ShopPackageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ShopSetting" (
    "ShopSettingId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "ReceiptHeader" varchar(200),
    "InVat" bool NOT NULL,
    "TaxRate" numeric NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "VatSystem" bool NOT NULL DEFAULT false,
    "PriceSystem" varchar(20),
    "CleanFixDuration" int4 NOT NULL DEFAULT 0,
    PRIMARY KEY ("ShopSettingId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."ShopTrial" (
    "ShopTrialId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "Package" varchar(40),
    "StartDateTime" timestamp NOT NULL,
    "EndDateTime" timestamp NOT NULL,
    "Description" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "State" bool NOT NULL,
    "UpdateBy" varchar(40),
    "Current" bool NOT NULL DEFAULT false,
    PRIMARY KEY ("ShopTrialId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Size" (
    "SizeId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Size_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("SizeId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SlipImage" (
    "SlipImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "OrderHeaderId" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("SlipImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Stock" (
    "StockId" varchar(50) NOT NULL,
    "BranchId" varchar(50),
    "ProductItemId" varchar(50),
    "InStock" int4 NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "ProductGroupId" varchar(50),
    "Code" varchar(40),
    CONSTRAINT "FK_Stock_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    CONSTRAINT "FK_Stock_ProductGroup_ProductGroupId" FOREIGN KEY ("ProductGroupId") REFERENCES "public"."ProductGroup"("ProductGroupId") ON DELETE RESTRICT,
    CONSTRAINT "FK_Stock_ProductItem_ProductItemId" FOREIGN KEY ("ProductItemId") REFERENCES "public"."ProductItem"("ProductItemId") ON DELETE RESTRICT,
    CONSTRAINT "FK_Stock_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("StockId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."SystemConfig" (
    "SystemConfigId" varchar(50) NOT NULL,
    "Name" varchar(100) NOT NULL,
    "Description" varchar(255),
    "ValueEN" varchar(100),
    "ValueTH" varchar(100),
    "ShopInfoId" text,
    PRIMARY KEY ("SystemConfigId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Test" (
    "TestId" varchar(50) NOT NULL,
    "Code" varchar(40),
    "Name" varchar(100),
    "Description" varchar(255),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "ShopInfoId" text,
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "TestText" text,
    PRIMARY KEY ("TestId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."TestImage" (
    "TestImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "OrderHeaderId" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("TestImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Texture" (
    "TextureId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Texture_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("TextureId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."User" (
    "UserId" text NOT NULL,
    "FirstName" text,
    "LastName" text,
    "Username" text,
    "UserImage" text,
    "Mobile" text,
    "Phone" text,
    "Email" text,
    "PasswordHash" bytea,
    "PasswordSalt" bytea,
    "Token" text,
    "TokenExpire" timestamp NOT NULL,
    "RefreshToken" text,
    "Active" bool NOT NULL,
    "Status" text,
    "ShopInfoId" varchar(50),
    "CreateBy" text,
    "RegisterToken" text,
    "RegisterTokenActive" bool NOT NULL,
    "RegisterTokenExpire" timestamp NOT NULL,
    "UpdateBy" text,
    "CreateDate" timestamp NOT NULL,
    "UpdateDate" timestamp NOT NULL,
    CONSTRAINT "FK_User_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("UserId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."UserBranch" (
    "UserBranchId" text NOT NULL,
    "UserId" text,
    "BranchId" varchar(50),
    CONSTRAINT "FK_UserBranch_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    CONSTRAINT "FK_UserBranch_User_UserId" FOREIGN KEY ("UserId") REFERENCES "public"."User"("UserId") ON DELETE RESTRICT,
    PRIMARY KEY ("UserBranchId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."UserRole" (
    "UserRoleId" text NOT NULL,
    "UserId" text,
    "RoleId" text,
    CONSTRAINT "FK_UserRole_Role_RoleId" FOREIGN KEY ("RoleId") REFERENCES "public"."Role"("RoleId") ON DELETE RESTRICT,
    CONSTRAINT "FK_UserRole_User_UserId" FOREIGN KEY ("UserId") REFERENCES "public"."User"("UserId") ON DELETE RESTRICT,
    PRIMARY KEY ("UserRoleId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Voucher" (
    "VoucherId" varchar(50) NOT NULL,
    "ShopInfoId" varchar(50),
    "Name" varchar(100),
    "Code" varchar(40),
    "InStock" int4 NOT NULL,
    "Month" int4 NOT NULL,
    "Price" numeric NOT NULL,
    "Value" numeric NOT NULL,
    "MainImageId" varchar(50),
    "Description" varchar(255),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "Deleted" bool NOT NULL,
    "DeleteDateTime" timestamp NOT NULL,
    "DeleteBy" varchar(40),
    CONSTRAINT "FK_Voucher_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("VoucherId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."VoucherImage" (
    "VoucherImageId" varchar(50) NOT NULL,
    "FileName" varchar(255) NOT NULL,
    "FileType" varchar(255) NOT NULL,
    "UploadFileName" varchar(255) NOT NULL,
    "PathName" varchar(255) NOT NULL,
    "Description" varchar(300),
    "VoucherId" varchar(50) NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" text,
    CONSTRAINT "FK_VoucherImage_Voucher_VoucherId" FOREIGN KEY ("VoucherId") REFERENCES "public"."Voucher"("VoucherId") ON DELETE CASCADE,
    PRIMARY KEY ("VoucherImageId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WebCategory" (
    "WebCategoryId" varchar(50) NOT NULL,
    "Code" varchar(10),
    "Description" varchar(50),
    "State" bool NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_WebCategory_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("WebCategoryId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WebClient" (
    "WebClientId" varchar(50) NOT NULL,
    "ClientId" varchar(100),
    "ClientSecret" varchar(100),
    "Token1" varchar(100),
    "Token2" varchar(100),
    "ShopInfoId" varchar(50),
    "ShopName" varchar(200),
    "Description" varchar(400),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "OrderStatus" varchar(100),
    "Active" bool NOT NULL,
    "UpdateBy" varchar(40),
    PRIMARY KEY ("WebClientId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WknOrder" (
    "WknOrderId" varchar(50) NOT NULL,
    "OrderNumber" varchar(20),
    "BranchId" varchar(50),
    "OrderStatus" varchar(50),
    "Approved" bool NOT NULL,
    "Cancel" bool NOT NULL,
    "CustomerId" varchar(50),
    "CustomerName" varchar(100),
    "CustomerAddress" varchar(500),
    "CustomerPhone" varchar(30),
    "LineId" text,
    "BirthDate" timestamp NOT NULL,
    "HowToKnowShop" text,
    "AccountNumber" varchar(100),
    "AccountName" varchar(100),
    "BankName" varchar(100),
    "StartDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "PaymentMethod" text,
    "RentalPriceTotal" numeric NOT NULL,
    "RentalDiscount" numeric NOT NULL,
    "NetPrice" numeric NOT NULL,
    "BailTotal" numeric NOT NULL,
    "BailDiscount" numeric NOT NULL,
    "BailNetTotal" numeric NOT NULL,
    "GrandTotal" numeric NOT NULL,
    "Vat" numeric NOT NULL,
    "BeforeVat" numeric NOT NULL,
    "InVat" bool NOT NULL,
    "VatSystem" bool NOT NULL,
    "TaxRate" numeric NOT NULL,
    "Description" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "OrderHeaderId" varchar(50),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_WknOrder_OrderHeader_OrderHeaderId" FOREIGN KEY ("OrderHeaderId") REFERENCES "public"."OrderHeader"("OrderHeaderId") ON DELETE RESTRICT,
    CONSTRAINT "FK_WknOrder_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("WknOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WknOrderDetail" (
    "WknOrderDetailId" varchar(50) NOT NULL,
    "ProductItemId" varchar(50),
    "WknOrderId" varchar(50),
    "BranchId" varchar(50),
    "MainImage" text,
    "ReservedQuantity" int4 NOT NULL,
    "RentalPrice" numeric NOT NULL,
    "Bail" numeric NOT NULL,
    "StartDate" timestamp NOT NULL,
    "EndDate" timestamp NOT NULL,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_WknOrderDetail_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_WknOrderDetail_WknOrder_WknOrderId" FOREIGN KEY ("WknOrderId") REFERENCES "public"."WknOrder"("WknOrderId") ON DELETE RESTRICT,
    PRIMARY KEY ("WknOrderDetailId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WnfOrder" (
    "WnfOrderId" varchar(50) NOT NULL,
    "WnfShopId" varchar(50),
    "OrderNumber" varchar(20),
    "OrderStatus" varchar(50),
    "Remark" varchar(500),
    "SendingDate" timestamp NOT NULL,
    "ReceivingDate" timestamp NOT NULL,
    "BarcodeImage" text,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "BranchId" varchar(50),
    CONSTRAINT "FK_WnfOrder_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    CONSTRAINT "FK_WnfOrder_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    CONSTRAINT "FK_WnfOrder_WnfShop_WnfShopId" FOREIGN KEY ("WnfShopId") REFERENCES "public"."WnfShop"("WnfShopId") ON DELETE RESTRICT,
    PRIMARY KEY ("WnfOrderId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."WnfShop" (
    "WnfShopId" varchar(50) NOT NULL,
    "Name" varchar(50),
    "Address" varchar(500),
    "Phone" varchar(50),
    "Status" bool NOT NULL,
    "Description" varchar(500),
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    "BranchId" varchar(50),
    CONSTRAINT "FK_WnfShop_Branch_BranchId" FOREIGN KEY ("BranchId") REFERENCES "public"."Branch"("BranchId") ON DELETE RESTRICT,
    CONSTRAINT "FK_WnfShop_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("WnfShopId")
);

-- This script only contains the table creation statements and does not fully represent the table in the database. It's still missing: indices, triggers. Do not use it as a backup.

-- Table Definition
CREATE TABLE "public"."Year" (
    "YearId" varchar(50) NOT NULL,
    "YearDc" int4,
    "YearOrder" int4,
    "Code3Digit" int4,
    "CreateDateTime" timestamp NOT NULL,
    "CreateBy" varchar(40),
    "UpdateDateTime" timestamp NOT NULL,
    "UpdateBy" varchar(40),
    "ShopInfoId" varchar(50),
    CONSTRAINT "FK_Year_ShopInfo_ShopInfoId" FOREIGN KEY ("ShopInfoId") REFERENCES "public"."ShopInfo"("ShopInfoId") ON DELETE RESTRICT,
    PRIMARY KEY ("YearId")
);

